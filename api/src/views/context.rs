//! This module holds contextual data that gets carried with each request. The
//! top-level struct is [RequestContext].

use crate::{
    error::{ClientError, ResponseResult},
    models,
    schema::{user_providers, users},
    util::PooledConnection,
};
use diesel::{
    NullableExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl,
};
use uuid::Uuid;

/// Information on the logged-in user.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorizedUser {
    pub id: Uuid,
    pub username: String,
}

impl AuthorizedUser {
    /// Convert this user into the user model struct.
    pub fn into_model(self) -> models::User {
        models::User {
            id: self.id,
            username: self.username,
        }
    }
}

/// A nested part of context, representing the authenticated user.
#[derive(Clone, Debug, PartialEq)]
pub struct UserContext {
    /// The ID of the user_provider that the user used to log in.
    pub user_provider_id: Uuid,
    /// Information on the requesting user. None if they are logged in but have
    /// not set their username yet.
    pub user: Option<AuthorizedUser>,
}

impl UserContext {
    /// Load the user context from the database. The user_provider ID should
    /// come from some authentication.
    fn load_context(
        conn: &PgConnection,
        user_provider_id: Uuid,
    ) -> ResponseResult<Option<Self>> {
        // TODO after diesel 2.0, merge these two queries into one

        // This is a double option for a reason - the outer option indicates
        // if the user_providers row exists in the DB. The inner option
        // indicates if the user_id column is populated in that row.
        // The outer should usually be Some if we get this far, it only
        // wouldn't be if that user_providers row has been deleted but the
        // cookie hasn't expired yet.
        // The inner should only be None if the user has logged in, but
        // not set their username yet, so that a row in the users table
        // hasn't been created yet.
        let user_query_result: Option<(Option<Uuid>, Option<String>)> =
            user_providers::table
                .find(user_provider_id)
                .left_join(users::table)
                .select((
                    users::columns::id.nullable(),
                    users::columns::username.nullable(),
                ))
                .get_result(conn)
                .optional()?;

        let user_context = match user_query_result {
            // The cookie is invalid, so user isn't logged in
            None => None,
            // If the inner value is none, then the user is logged in but
            // not initialized
            Some((None, None)) => Some(Self {
                user_provider_id,
                user: None,
            }),
            // User is logged in and initialized, return the full user data
            Some((Some(user_id), Some(username))) => Some(Self {
                user_provider_id,
                user: Some(AuthorizedUser {
                    id: user_id,
                    username,
                }),
            }),
            // The user ID and username should always be either both None or
            // both Some, since they can both only be None when the left_join
            // gives a null user. Both columns are NOT NULL so if the join
            // matches something, then they should both be defined.
            Some((None, Some(_))) | Some((Some(_), None)) => {
                panic!("Unexpected query result: {:?}", user_query_result)
            }
        };

        Ok(user_context)
    }
}

/// The context that gets attached to every incoming request.
pub struct RequestContext {
    /// DB connection
    pub db_conn: PooledConnection,
    /// The authenticated user. None if the requesting user is not logged in.
    pub user_context: Option<UserContext>,
}

impl RequestContext {
    /// Load the context from the given request inputs.
    pub fn load_context(
        db_conn: PooledConnection,
        user_provider_id: Option<Uuid>,
    ) -> ResponseResult<Self> {
        // Load user context for the given user_providers ID
        let user_context: Option<UserContext> = match user_provider_id {
            None => None,
            Some(upid) => UserContext::load_context(&db_conn, upid)?,
        };

        Ok(Self {
            db_conn,
            user_context,
        })
    }

    pub fn db_conn(&self) -> &PgConnection {
        &self.db_conn
    }

    /// Get the ID of the authenticated user. If the user isn't authenticated,
    /// or they ARE authenticated but haven't finished initializing their user
    /// yet (to the point where a row in the `users` table has been created),
    /// then return an error.
    pub fn user(&self) -> ResponseResult<&AuthorizedUser> {
        match &self.user_context {
            Some(UserContext {
                user: Some(user), ..
            }) => Ok(&user),
            _ => Err(ClientError::Unauthenticated.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::Factory, util};

    /// Test when the given user provider ID isn't in the DB
    #[test]
    fn test_user_load_context_invalid_user_provider() {
        let pool = util::init_test_db_conn_pool().unwrap();
        let ctx = UserContext::load_context(&pool.get().unwrap(), Uuid::nil());
        assert_eq!(ctx.unwrap(), None);
    }

    /// Test when the user has a user_provider but the user hasn't been
    /// initialized yet.
    #[test]
    fn test_user_load_context_uninitialized_user() {
        let pool = util::init_test_db_conn_pool().unwrap();
        let conn = &pool.get().unwrap();
        let user_provider = models::NewUserProvider {
            sub: "sub",
            provider_name: "provider",
            user_id: None,
        }
        .create(conn);

        let ctx = UserContext::load_context(conn, user_provider.id);
        assert_eq!(
            ctx.unwrap(),
            Some(UserContext {
                user_provider_id: user_provider.id,
                user: None
            })
        );
    }

    #[test]
    fn test_user_load_context_initialized_user() {
        let pool = util::init_test_db_conn_pool().unwrap();
        let conn = &pool.get().unwrap();

        let user = models::NewUser { username: "user1" }.create(conn);
        let user_provider = models::NewUserProvider {
            sub: "sub",
            provider_name: "provider",
            user_id: Some(user.id),
        }
        .create(conn);

        let ctx = UserContext::load_context(conn, user_provider.id);
        assert_eq!(
            ctx.unwrap(),
            Some(UserContext {
                user_provider_id: user_provider.id,
                user: Some(AuthorizedUser {
                    id: user.id,
                    username: user.username
                })
            })
        );
    }
}
