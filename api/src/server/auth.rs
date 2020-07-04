use crate::{
    config::{OpenIdConfig, ProviderConfig},
    error::ResponseError,
    models::{NewUser, NewUserProvider, User, UserProvider},
    schema::{user_providers, users},
    util::Pool,
};
use actix_identity::Identity;
use actix_web::{get, http, web, HttpResponse};
use diesel::{
    prelude::RunQueryDsl, ExpressionMethods, OptionalExtension, PgConnection,
    QueryDsl,
};
use openid::{Client, DiscoveredClient, Options, Token, Userinfo};
use reqwest::Url;
use serde::Deserialize;
use std::collections::HashMap;

/// Map of provider name to configured [Client]
pub struct ClientMap {
    pub map: HashMap<String, Client>,
}

impl ClientMap {
    pub fn get_client(
        &self,
        provider_name: &str,
    ) -> Result<&Client, HttpResponse> {
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
    ) -> Client {
        let redirect = Some(format!("{}/api/oidc/{}/callback", host_url, name));
        let issuer = Url::parse(&provider_config.issuer_url).unwrap();
        DiscoveredClient::discover(
            provider_config.client_id.clone(),
            provider_config.client_secret.clone(),
            redirect,
            issuer,
        )
        .await
        .unwrap()
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
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: handle bad name
    let provider_name = params.0.to_string();
    let oidc_client = client_map.get_client(&provider_name)?;

    let auth_url = oidc_client.auth_url(&Options {
        scope: Some("email".into()),
        ..Default::default()
    });

    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, auth_url.to_string())
        .finish())
}

#[derive(Deserialize, Debug)]
pub struct LoginQuery {
    code: String,
}

/// Exchanges the access token from the initial login in the openid provider
/// for a normal token. The code here should come from the browser, which
/// is passed along from the provider.
async fn request_token(
    oidc_client: &Client,
    code: &str,
) -> Result<(Token, Userinfo), ResponseError> {
    let mut token: Token = oidc_client.request_token(&code).await?.into();
    if let Some(mut id_token) = token.id_token.as_mut() {
        // Decode the JWT and validate it was signed by the provider
        oidc_client.decode_token(&mut id_token)?;
        oidc_client.validate_token(&id_token, None, None)?;

        // Call to the userinfo endpoint of the provider
        let userinfo = oidc_client.request_userinfo(&token).await?;
        Ok((token, userinfo))
    } else {
        Err(ResponseError::InvalidCredentials)
    }
}

/// Called when a [User] does not yet exist for the [UserProvider].
/// Will redirect to a page to set their username to make the [User].
fn init_user(
    user_provider: &UserProvider,
    userinfo: Userinfo,
    conn: &PgConnection,
) -> Result<(), ResponseError> {
    // Insert the user into the DB
    let user: User = NewUser {
        // TODO: for now this will set username automatically but
        // we will need to redirect them to set it before making the
        // user
        username: &userinfo.email.unwrap()[..20],
    }
    .insert()
    .returning(users::all_columns)
    .get_result(conn)
    .map_err(ResponseError::from)?;

    // update provider with the user id
    diesel::update(
        user_providers::table
            .filter(user_providers::dsl::id.eq(user_provider.id)),
    )
    .set(user_providers::columns::user_id.eq(user.id))
    .execute(conn)
    .map_err(ResponseError::from)?;

    Ok(())
}

/// Logs the [User] in by setting a cookie based on the [UserProvider] id.
fn log_in_user(
    user_provider: &UserProvider,
    identity: &Identity,
) -> HttpResponse {
    // Adds a cookie which can be used to auth requests. We use the UserProvider
    // ID so that this works even if the User object hasn't been created yet.
    identity.remember(user_provider.id.to_string());

    // TODO: add redirect path to state param so we don't always
    // redirect to the homepage
    HttpResponse::Found()
        .header(http::header::LOCATION, "/")
        .finish()
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

    // Send the user's code to the server to authenticate it
    let (_, userinfo) = request_token(oidc_client, &query.code).await?;

    // Not sure when this can be None, hopefully never??
    let sub: &str = userinfo.sub.as_ref().unwrap();
    let existing_user_provider =
        UserProvider::filter_by_sub_and_provider(sub, &provider_name)
            .get_result::<UserProvider>(conn)
            .optional()
            .map_err(ResponseError::from)?;
    match existing_user_provider {
        Some(user_provider) => match user_provider.user_id {
            Some(_) => {
                // User already exists so normal login
                Ok(log_in_user(&user_provider, &identity))
            }
            None => {
                // no user account associated with this user_provider
                // yet so make one (they have logged in but did not set
                // username)
                init_user(&user_provider, userinfo, conn)?;
                Ok(log_in_user(&user_provider, &identity))
            }
        },
        None => {
            // user_provider not found (first login) so make the row
            // then init the user
            let user_provider: UserProvider = NewUserProvider {
                sub,
                provider_name,
                user_id: None,
            }
            .insert()
            .returning(user_providers::all_columns)
            .get_result(conn)
            .unwrap();

            // Create a new User object, then set the auth cookie
            init_user(&user_provider, userinfo, conn)?;
            Ok(log_in_user(&user_provider, &identity))
        }
    }
}
