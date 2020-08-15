use crate::{
    config::{OpenIdConfig, ProviderConfig},
    error::ResponseError,
    models::NewUserProvider,
    schema::user_providers,
    util::{self, Pool},
};
use actix_identity::Identity;
use actix_web::{get, http, post, web, HttpResponse};
use diesel::{
    Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl,
};
use openidconnect::{
    core::{CoreClient, CoreProviderMetadata, CoreResponseType},
    AuthenticationFlow, ClientId, ClientSecret, IssuerUrl, Nonce, RedirectUrl,
    Scope,
};
use serde::Deserialize;
use std::collections::HashMap;
use util::AuthState;
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

    // Build a client for each provider
    // TODO do these in parallel
    let mut map = HashMap::new();
    for (name, provider_config) in &open_id_config.providers {
        let client = make_client(host_url, name, provider_config).await;
        map.insert(name.into(), client);
    }

    ClientMap { map }
}

/// The frontend will redirect to this before being sent off to the
/// actual openid provider
#[get("/api/oidc/{provider_name}/redirect")]
pub async fn route_authorize(
    client_map: web::Data<ClientMap>,
    params: web::Path<(String,)>,
    query: web::Query<RedirectQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let provider_name: &str = &params.0;
    let oidc_client = client_map.get_client(provider_name)?;
    let next = query.next.clone();

    let (auth_url, _csrf_state, _nonce) = oidc_client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            move || AuthState::new(next).serialize(),
            || Nonce::new("4".into()),
        )
        .add_scope(Scope::new("email".to_string()))
        // Serialization shouldn't ever fail so yeet that shit outta the Result
        .add_extra_param("next", query.next.as_deref().unwrap_or(""))
        .url();

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
    let conn = &pool.get().map_err(ResponseError::from)? as &PgConnection;

    // Parse the state param and validate the CSRF token in there
    let auth_state = AuthState::deserialize(query.state.as_deref())?;

    // Send the user's code to the server to authenticate it
    let user_info = util::oidc_request_token(oidc_client, &query.code).await?;

    // Not sure when this can be None, hopefully never??
    let sub: &str = user_info.subject().as_str();

    // Insert the sub+provider, or just return the existing one if it's already
    // in the DB. We need to do this in a transaction to prevent race conditions
    // if the provider gets deleted in another thread.
    let user_provider_id: Uuid =
        conn.transaction::<Uuid, ResponseError, _>(|| {
            // Insert, if the row already exists, just return None
            let inserted = NewUserProvider {
                sub,
                provider_name,
                user_id: None,
            }
            .insert()
            .on_conflict_do_nothing()
            .returning(user_providers::columns::id)
            .get_result(conn)
            .optional()
            .map_err(ResponseError::from)?;

            match inserted {
                // Insert didn't return anything, which means the row is already
                // in the DB. Just select that row.
                None => user_providers::table
                    .select(user_providers::columns::id)
                    .filter(user_providers::columns::sub.eq(sub))
                    .filter(
                        user_providers::columns::provider_name
                            .eq(provider_name),
                    )
                    .get_result(conn)
                    .map_err(ResponseError::from),
                Some(inserted_id) => Ok(inserted_id),
            }
        })?;

    // Add a cookie which can be used to auth requests. We use the UserProvider
    // ID so that this works even if the User object hasn't been created yet.
    identity.remember(user_provider_id.to_string());

    // Redirect to the path specified in the OpenID state param
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, auth_state.next())
        .finish())
}

#[post("/api/logout")]
pub async fn logout_route(identity: Identity) -> HttpResponse {
    identity.forget();
    HttpResponse::Ok().finish()
}
