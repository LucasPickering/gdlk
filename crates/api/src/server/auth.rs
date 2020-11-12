use crate::{
    config::{OpenIdConfig, ProviderConfig},
    error::{ApiError, ClientError},
    models::NewUserProvider,
    schema::user_providers,
    util::{self, IdentityState, Pool},
};
use actix_identity::Identity;
use actix_web::{get, http, post, web, HttpResponse};
use diesel::{
    ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl,
};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    AuthenticationFlow, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

/// Map of provider name to configured [CoreClient]
pub struct ClientMap {
    pub map: HashMap<String, CoreClient>,
}

#[derive(Deserialize, Debug)]
pub struct RedirectQuery {
    next: Option<String>,
}

impl ClientMap {
    pub fn get_client(
        &self,
        provider_name: &str,
    ) -> Result<&CoreClient, HttpResponse> {
        self.map
            .get(provider_name)
            .ok_or_else(|| HttpResponse::NotFound().finish())
    }
}

/// Build a map of OpenID clients, one for each provider.
pub async fn build_client_map(open_id_config: &OpenIdConfig) -> ClientMap {
    async fn make_client(
        host_url: &str,
        name: &str,
        provider_config: &ProviderConfig,
    ) -> CoreClient {
        let redirect = RedirectUrl::new(format!(
            "{}/api/oidc/{}/callback",
            host_url, name
        ))
        .expect("Invalid redirect URL");
        let issuer =
            IssuerUrl::new(provider_config.issuer_url.clone()).unwrap();

        let metadata = CoreProviderMetadata::discover_async(
            issuer,
            util::oidc_http_client,
        )
        .await
        .unwrap();

        CoreClient::from_provider_metadata(
            metadata,
            ClientId::new(provider_config.client_id.clone()),
            Some(ClientSecret::new(provider_config.client_secret.clone())),
        )
        .set_redirect_uri(redirect)
    }

    let host_url: &str = &open_id_config.host_url;

    // Create one future per provider
    let futures = open_id_config.providers.iter().map(
        async move |(name, provider_config)| {
            (
                name.to_owned(),
                make_client(host_url, name, provider_config).await,
            )
        },
    );
    // Resolve all the futures (concurrently), then collect results into a map
    let map: HashMap<_, _> = futures::future::join_all(futures)
        .await
        .into_iter()
        .collect();

    ClientMap { map }
}

/// The frontend will redirect to this before being sent off to the
/// actual openid provider
#[get("/api/oidc/{provider_name}/redirect")]
pub async fn route_authorize(
    client_map: web::Data<ClientMap>,
    params: web::Path<(String,)>,
    query: web::Query<RedirectQuery>,
    identity: Identity,
) -> Result<HttpResponse, actix_web::Error> {
    let provider_name: &str = &params.0;
    let oidc_client = client_map.get_client(provider_name)?;

    let (auth_url, csrf_token, _nonce) = oidc_client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            // TODO use a real nonce
            // https://github.com/LucasPickering/gdlk/issues/159
            || Nonce::new("4".into()),
        )
        .add_scope(Scope::new("email".to_string()))
        .url();

    // Encode the CSRF token and some extra data, then store that in a
    // signed+encrypted cookie. We'll read the CSRF token from there in the
    // callback and compare to what we get from the URL. Since the cookie is
    // signed, this prevents CSRF attacks.
    let state = IdentityState::DuringAuth {
        next: query.next.clone(),
        csrf_token,
    };
    identity.remember(state.serialize());

    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, auth_url.to_string())
        .finish())
}

#[derive(Deserialize, Debug)]
pub struct LoginQuery {
    code: String,
    state: Option<String>,
    nonce: Option<String>,
}

/// Provider redirects back to this route after the login
#[get("/api/oidc/{provider_name}/callback")]
pub async fn route_login(
    client_map: web::Data<ClientMap>,
    params: web::Path<(String,)>,
    query: web::Query<LoginQuery>,
    pool: web::Data<Pool>,
    identity: Identity,
) -> Result<HttpResponse, actix_web::Error> {
    let provider_name: &str = &params.0;
    let oidc_client = client_map.get_client(provider_name)?;
    let conn =
        &pool.get().map_err(ApiError::from_server_error)? as &PgConnection;
    // Read identity/state data that stored in an encrypted+signed cookie. We
    // know this data is safe, we wrote it and it hasn't been modified.
    let identity_state =
        IdentityState::from_identity(&identity).ok_or_else(|| {
            ApiError::from_client_error(ClientError::Unauthenticated)
        })?;

    // VERY IMPORTANT - read the CSRF token from the state param, and compare it
    // to the token we stored in the cookie. The cookie is encrypted+signed,
    // Parse the state param and validate the CSRF token in there
    identity_state.verify_csrf(query.state.as_deref().unwrap_or(""))?;

    // Send the user's code to the server to authenticate it
    let user_info = util::oidc_request_token(oidc_client, &query.code).await?;
    let sub: &str = user_info.subject().as_str();

    // In most cases, the user should already hvae a user_provider row. If not,
    // insert one.
    // Note: there is a potential race condition here, if this is called twice
    // simultaneously for a new user. Both could try to insert the row, in which
    // case the 2nd insert will fail, triggering a 500. It should still leave
    // the DB in a valid state tho and is extremely unlikely, so not worth the
    // perf impact of a transaction.
    let user_provider_id: Uuid = {
        let existing_id = user_providers::table
            .select(user_providers::columns::id)
            .filter(user_providers::columns::sub.eq(sub))
            .filter(user_providers::columns::provider_name.eq(provider_name))
            .get_result(conn)
            .optional()
            .map_err(ApiError::from_server_error)?;

        match existing_id {
            // user_provider doesn't exist, add a new one
            None => NewUserProvider {
                sub,
                provider_name,
                user_id: None,
            }
            .insert()
            .returning(user_providers::columns::id)
            .get_result(conn)
            .map_err(ApiError::from_server_error)?,
            Some(existing_id) => existing_id,
        }
    };

    // Replace the auth state cookie with one for permanenet auth. We use the
    // UserProvider ID so that this works even if the User object hasn't
    // been created yet.
    let new_identity_state = IdentityState::PostAuth { user_provider_id };
    identity.remember(new_identity_state.serialize());

    // Redirect to the path specified in the state cookie
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, identity_state.next())
        .finish())
}

#[post("/api/logout")]
pub async fn logout_route(identity: Identity) -> HttpResponse {
    identity.forget();
    HttpResponse::Ok().finish()
}
