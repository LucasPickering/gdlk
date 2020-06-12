//! All code related to the webserver. Basically anything that calls Actix
//! lives here.
mod auth;
mod gql;
pub use crate::server::gql::{create_gql_schema, Context, GqlSchema};
use crate::{
    server::auth::{
        read_config, route_authorize, route_login, ClientMap, Sessions,
    },
    util::Pool,
};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
};
#[get("/api/graphiql")]
async fn route_graphiql() -> HttpResponse {
    let html = graphiql_source("/api/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[post("/api/graphql")]
async fn route_graphql(
    pool: web::Data<Pool>,
    st: web::Data<Arc<GqlSchema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = web::block(move || {
        let res = data.execute(
            &st,
            &Context {
                pool: pool.into_inner(),
            },
        );
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

#[actix_rt::main]
pub async fn run_server(pool: Pool, host: String) -> io::Result<()> {
    // Init GraphQL schema
    let gql_schema = Arc::new(create_gql_schema());

    // let google_client_id = env::var("OPENID_CLIENT_ID").unwrap();
    // let google_client_secret = env::var("OPENID_CLIENT_SECRET").unwrap();
    // let google_url = "https://accounts.google.com";
    // let client =
    //     make_client(google_client_id, google_client_secret,
    // google_url).await; let client = web::Data::new(client);
    let mut client_map = ClientMap {
        map: HashMap::new(),
    };
    read_config("openid_config.json", &mut client_map)
        .await
        .unwrap();
    let client_map = web::Data::new(client_map);
    let sessions = web::Data::new(RwLock::new(Sessions {
        map: HashMap::new(),
    }));

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            // Need to clone these because init occurs once per thread
            .data(pool.clone())
            .data(gql_schema.clone())
            .app_data(client_map.clone())
            .app_data(sessions.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-openid")
                    .secure(false), // TODO: be secure
            ))
            // routes
            .service(route_graphql)
            .service(route_graphiql)
            .service(route_login)
            .service(route_authorize)
    })
    .bind(host)?
    .run()
    .await
}
