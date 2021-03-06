//! All code related to the webserver. Basically anything that calls Actix
//! lives here.
mod auth;
mod gql;

pub use crate::server::gql::{create_gql_schema, GqlSchema};
use crate::{
    config::GdlkConfig,
    server::auth::{logout_route, route_authorize, route_login},
    util::{self, IdentityState, Pool},
    views::RequestContext,
};
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{
    cookie::SameSite, get, middleware, post, web, App, HttpResponse, HttpServer,
};
use chrono::Duration;
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use std::io;
use uuid::Uuid;

#[get("/api/graphiql")]
async fn route_graphiql() -> HttpResponse {
    let html = graphiql_source("/api/graphql", None);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[post("/api/graphql")]
async fn route_graphql(
    identity: Identity,
    pool: web::Data<Pool>,
    gql_schema: web::Data<GqlSchema>,
    gql_request: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // Auth cookie holds a user provider ID - if present, the user is logged in
    let identity_state = IdentityState::from_identity(&identity);
    let user_provider_id: Option<Uuid> = identity_state
        .map(|id_state| id_state.user_provider_id())
        .flatten();

    let context =
        RequestContext::load_context(pool.into_inner(), user_provider_id)?;
    let response = gql_request.execute(&gql_schema, &context).await;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&response)?))
}

#[actix_rt::main]
pub async fn run_server(config: GdlkConfig) -> io::Result<()> {
    // Initialize env shit
    let pool = util::init_db_conn_pool(&config.database_url).unwrap();
    let gql_schema = web::Data::new(create_gql_schema());
    let client_map =
        web::Data::new(auth::build_client_map(&config.open_id).await);
    let secret_key: Vec<u8> = base64::decode(&config.secret_key).unwrap();

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            // Need to clone these because init occurs once per thread
            .data(pool.clone())
            .app_data(gql_schema.clone())
            .app_data(client_map.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&secret_key)
                    .name("auth-openid")
                    .same_site(SameSite::Lax) // Prevent CSRF
                    .secure(true) // Only send cookie over HTTPS
                    .max_age_time(Duration::days(1)),
            ))
            // routes
            .service(route_graphql)
            .service(route_graphiql)
            .service(route_login)
            .service(logout_route)
            .service(route_authorize)
    })
    .bind(&config.server_host)?
    .run()
    .await
}
