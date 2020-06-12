use crate::{
    error::ResponseError,
    models::{NewUser, User},
    schema::users,
    util::Pool,
};
use actix_identity::Identity;
use actix_web::{get, http, web, HttpResponse, Responder};
use diesel::{prelude::RunQueryDsl, PgConnection};
use failure::Fallible;
use openid::{Client, DiscoveredClient, Options, Token, Userinfo};
use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashMap, fs, sync::RwLock};

// TODO: make this a db table so its persistent through restarts
pub struct Sessions {
    pub map: HashMap<String, (User, Token, Userinfo)>,
}

pub struct ClientMap {
    pub map: HashMap<String, Client>,
}

#[derive(Deserialize)]
struct OpenidConfig {
    host_url: String,
    providers: Vec<ProviderConfig>,
}

#[derive(Deserialize)]
struct ProviderConfig {
    name: String,
    client_id: String,
    client_secret: String,
    issuer_url: String,
}

pub async fn read_config(
    path: &str,
    client_map: &mut ClientMap,
) -> Fallible<()> {
    let config_str = fs::read_to_string(path)?;
    let openid_config: OpenidConfig = serde_json::from_str(&config_str)?;
    init_config(openid_config, client_map).await;
    Ok(())
}

async fn init_config(openid_config: OpenidConfig, client_map: &mut ClientMap) {
    for config in openid_config.providers {
        let client = make_client(
            config.client_id,
            config.client_secret,
            config.issuer_url,
            &config.name,
            &openid_config.host_url,
        )
        .await;

        client_map.map.insert(config.name, client);
    }
}

/// Setup a client for a given provider
/// based on the id, secret, and provider url
async fn make_client(
    client_id: String,
    client_secret: String,
    issuer_url: String,
    name: &str,
    host_url: &str,
) -> Client {
    let redirect = Some(format!("{}/api/oidc/{}/callback", host_url, name));
    let issuer = match Url::parse(&issuer_url) {
        Ok(res) => res,
        Err(e) => panic!(e),
    };

    match DiscoveredClient::discover(client_id, client_secret, redirect, issuer)
        .await
    {
        Ok(res) => res,
        Err(e) => panic!(e),
    }
}

/// The frontend will redirect to this before being sent off to the
/// actual openid provider
// TODO: add route param to say which provider to use
#[get("/api/oidc/{provider_name}/redirect")]
pub async fn route_authorize(
    client_map: web::Data<ClientMap>,
    params: web::Path<(String,)>,
) -> impl Responder {
    // TODO: handle bad name
    let provider_name = params.0.to_string();
    let oidc_client = client_map.map.get(&provider_name).unwrap();

    let auth_url = oidc_client.auth_url(&Options {
        scope: Some("email".into()),
        ..Default::default()
    });

    HttpResponse::Found()
        .header(http::header::LOCATION, auth_url.to_string())
        .finish()
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
    // TODO: handle bad name
    let provider_name = params.0.to_string();
    let oidc_client = client_map.map.get(&provider_name).unwrap();
    let conn = &pool.get().map_err(ResponseError::from)? as &PgConnection;

    match request_token(oidc_client, query).await {
        Ok(Some((token, userinfo))) => {
            let email: String = userinfo.email.clone().unwrap();
            // Make the user
            // TODO: handle setting username
            // TODO: handle second login, right now it will error
            let user: User = NewUser {
                username: &email.chars().take(20).collect::<String>(),
            }
            .insert()
            .returning(users::all_columns)
            .get_result(conn)
            .unwrap();
            let id = user.id.to_string();

            // Make the user's session
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
        _ => {
            // Invalid call to the callback
            // Could have been no/invalid JWT
            Ok(HttpResponse::Unauthorized().finish())
        }
    }
}
