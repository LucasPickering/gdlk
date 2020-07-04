//! All code related to the webserver. Basically anything that calls Actix
//! lives here.
mod auth;
mod gql;

pub use crate::server::gql::{
    create_gql_schema, Context, GqlSchema, UserContext,
};
use crate::{
    config::GdlkConfig,
    error::ResponseError,
    schema::user_providers,
    server::auth::{route_authorize, route_login},
    util::{self, Pool},
};
use actix_identity::{CookieIdentityPolicy, Identity, IdentityService};
use actix_web::{
    cookie::SameSite, get, middleware, post, web, App, HttpResponse, HttpServer,
};
use chrono::Duration;
use diesel::{OptionalExtension, PgConnection, QueryDsl, RunQueryDsl};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use std::{io, sync::Arc};
use uuid::Uuid;

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
    identity: Identity,
    gql_schema: web::Data<Arc<GqlSchema>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    // Auth cookie holds a user provider ID. Validate it, and look up the
    // corresponding user ID.
    let user_context: Option<UserContext> = match identity.identity() {
        None => None,
        Some(user_provider_id) => {
            let conn =
                &pool.get().map_err(ResponseError::from)? as &PgConnection;
            let user_provider_uuid = util::parse_uuid(&user_provider_id);

            // This is a double option for a reason - the outer option indicates
            // if the user_providers row exists in the DB. The inner option
            // indicates if the user_id column is populated in that row.
            // The outer should usually be Some if we get this far, it only
            // wouldn't be if that user_providers row has been deleted but the
            // cookie hasn't expired yet.
            // The inner should only be None if the user has logged in, but
            // not set their username yet, so that a row in the users table
            // hasn't been created yet.
            let user_id_opt_opt: Option<Option<Uuid>> = user_providers::table
                .find(user_provider_uuid)
                .select(user_providers::columns::user_id)
                .get_result(conn)
                .optional()
                .map_err(ResponseError::from)?;

            // If the outer option is None, just return None (because this
            // cookie is no longer valid). If the inner is None, then we can
            // return a context but without a user attached.
            user_id_opt_opt.map(|user_id_opt| UserContext {
                user_provider_id: user_provider_uuid,
                user_id: user_id_opt,
            })
        }
    };

    let response = web::block(move || {
        let res = data.execute(
            &gql_schema,
            &Context {
                pool: pool.into_inner(),
                user_context,
            },
        );
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(response))
}

#[actix_rt::main]
pub async fn run_server(config: GdlkConfig, pool: Pool) -> io::Result<()> {
    // Init GraphQL schema
    let gql_schema = Arc::new(create_gql_schema());
    let client_map =
        web::Data::new(auth::build_client_map(&config.open_id).await);
    let secret_key: Vec<u8> = base64::decode(&config.secret_key).unwrap();

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            // Need to clone these because init occurs once per thread
            .data(pool.clone())
            .data(gql_schema.clone())
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
            .service(route_authorize)
    })
    .bind(&config.server_host)?
    .run()
    .await
}
