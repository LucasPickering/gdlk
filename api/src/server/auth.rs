use crate::{
    config::{OpenIdConfig, ProviderConfig},
    error::ResponseError,
    models::{NewUserProvider, User, UserProvider},
    schema::user_providers,
    util::Pool,
};
use actix_identity::Identity;
use actix_web::{get, http, web, HttpResponse};
use diesel::{prelude::RunQueryDsl, OptionalExtension, PgConnection};
use openid::{Client, DiscoveredClient, Options, Token, Userinfo};
use failure::Fallible;
use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashMap, sync::RwLock};

// TODO: make this a db table so its persistent through restarts
pub struct Sessions {
    pub map: HashMap<String, (User, Token, Userinfo)>,
}

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
/// for a normal token
async fn request_token(
    oidc_client: &Client,
    query: web::Query<LoginQuery>,
) -> Result<Option<(Token, Userinfo)>, actix_web::Error> {
    let mut token: Token = oidc_client
        .request_token(&query.code)
        .await
        .map_err(ResponseError::from)?
        .into();
    if let Some(mut id_token) = token.id_token.as_mut() {
        // Decode the JWT and validate it was signed by the provider
        oidc_client
            .decode_token(&mut id_token)
            .map_err(ResponseError::from)?;
        oidc_client
            .validate_token(&id_token, None, None)
            .map_err(ResponseError::from)?;
    } else {
        return Ok(None);
    }

    // Call to the userinfo endpoint of the provider
    let userinfo = oidc_client
        .request_userinfo(&token)
        .await
        .map_err(ResponseError::from)?;

    Ok(Some((token, userinfo)))
}

pub fn init_user(
    user_provider: &UserProvider,
    token: Token,
) -> Result<HttpResponse, actix_web::Error> {
}

pub fn login_user(
    user_provider: &UserProvider,
    token: Token,
    userinfo: Userinfo,
    identity: Identity,
    sessions: web::Data<RwLock<Sessions>>,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: do a join on user_providers table and users table
    // to get the user associated with this user_provider
    let id = user_provider.id.to_string();
    // Adds a cookie which can be used to auth requests
    identity.remember(id.clone());
    sessions
        .write()
        .unwrap()
        .map
        .insert(id, (user, token, userinfo));

    // TODO: add redirect path to state param so we don't always
    // redirect to the homepage
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, "/")
        .finish())
}

/// Provider redirects back to this route after the login
#[get("/api/oidc/{provider_name}/callback")]
pub async fn route_login(
    client_map: web::Data<ClientMap>,
    params: web::Path<(String,)>,
    query: web::Query<LoginQuery>,
    sessions: web::Data<RwLock<Sessions>>,
    pool: web::Data<Pool>,
    identity: Identity,
) -> Result<HttpResponse, actix_web::Error> {
    let provider_name = params.0.to_string();
    let oidc_client = client_map.get_client(&provider_name)?;

    let conn = &pool.get().map_err(ResponseError::from)? as &PgConnection;

    match request_token(oidc_client, query).await {
        Ok(Some((token, userinfo))) => {
            // let email: String = userinfo.email.clone().unwrap();
            let sub = userinfo.sub.clone().unwrap();

            let existing_user_provider =
                UserProvider::filter_by_sub_and_provider(&sub, &provider_name)
                    .get_result::<UserProvider>(conn)
                    .optional()
                    .map_err(ResponseError::from)?;
            match existing_user_provider {
                Some(user_provider) => match user_provider.user_id {
                    Some(user_id) => {
                        login_user(&user_provider, token, userinfo, identity)
                    }
                    None => {
                        // no user account associated with this user_provider
                        // yet so make one
                        init_user(&user_provider, token)
                    }
                },
                None => {
                    // user_provider not found so make the row then init the
                    // user
                    let user_provider: UserProvider = NewUserProvider {
                        sub: &sub,
                        provider_name: &provider_name,
                    }
                    .insert()
                    .returning(user_providers::all_columns)
                    .get_result(conn)
                    .unwrap();
                    init_user(&user_provider, token)
                }
            }
            // let id = user.id.to_string();

            // // Make the user's session
            // // Adds a cookie which can be used to auth requests
            // identity.remember(id.clone());
            // sessions
            //     .write()
            //     .unwrap()
            //     .map
            //     .insert(id, (user, token, userinfo));

            // TODO: add redirect path to state param so we don't always
            // redirect to the homepage
            // Ok(HttpResponse::Found()
            //     .header(http::header::LOCATION, "/")
            //     .finish())
        }
        _ => {
            // Invalid call to the callback
            // Could have been no/invalid JWT
            Ok(HttpResponse::Unauthorized().finish())
        }
    }
}
