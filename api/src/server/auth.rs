use crate::{
    error::ResponseError,
    models::{NewUser, User},
    schema::users,
    util::Pool,
};
use actix_identity::Identity;
use actix_web::{get, http, web, HttpResponse, Responder};
use diesel::{prelude::RunQueryDsl, PgConnection};
use openid::{Client, DiscoveredClient, Options, Token, Userinfo};
use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashMap, sync::RwLock};
// TODO: make this a db table so its persistent through restarts
pub struct Sessions {
    pub map: HashMap<String, (User, Token, Userinfo)>,
}
/// Setup a client for a given provider
/// based on the id, secret, and provider url
pub async fn make_client(
    client_id: String,
    client_secret: String,
    issuer_url: &str,
) -> Client {
    // TODO: read hostname from a config file
    let redirect = Some("localhost:3000/api/oidc/callback".to_string());

    let issuer = match Url::parse(issuer_url) {
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
#[get("/api/oidc/redirect")]
pub async fn route_authorize(
    oidc_client: web::Data<DiscoveredClient>,
) -> impl Responder {
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
    oidc_client: web::Data<DiscoveredClient>,
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
#[get("/api/oidc/callback")]
pub async fn route_login(
    oidc_client: web::Data<DiscoveredClient>,
    query: web::Query<LoginQuery>,
    sessions: web::Data<RwLock<Sessions>>,
    pool: web::Data<Pool>,
    identity: Identity,
) -> Result<HttpResponse, actix_web::Error> {
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
