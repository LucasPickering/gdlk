//! This module holds contextual data that gets carried with each request. The
//! top-level struct is [RequestContext].

use crate::{
    error::{ClientError, ResponseResult},
    models,
    schema::{
        permissions, role_permissions, roles, user_providers, user_roles, users,
    },
    util::PooledConnection,
};
use diesel::{
    NullableExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    RunQueryDsl,
};
use std::collections::HashSet;
use uuid::Uuid;

/// Information on the logged-in user.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthorizedUser {
    pub id: Uuid,
    pub username: String,
    pub permissions: HashSet<models::PermissionType>,
    pub is_admin: bool,
}

impl AuthorizedUser {
    /// Convert this user into the user model struct.
    pub fn into_model(self) -> models::User {
        models::User {
            id: self.id,
            username: self.username,
        }
    }

    /// Check if the user has the requested permission. Admins implicitly have
    /// all permissions, so this also checks `self.is_admin`.
    pub fn has_permission(&self, permission: models::PermissionType) -> bool {
        self.is_admin || self.permissions.contains(&permission)
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
        // TODO after diesel 2.0, merge these two queries into one.
        // Then we can do array/bool aggs for the permissions/is_admin.

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
            Some((Some(user_id), Some(username))) => {
                // Fetch all of the user's roles and permissions
                let permissions_query_result = users::table
                    .find(user_id)
                    // yo dawg i heard you like joins
                    .inner_join(
                        user_roles::table.left_join(
                            // LEFT JOIN here so that even if the role has
                            // no
                            // permissions, we still get its is_admin value
                            roles::table.left_join(
                                role_permissions::table
                                    .left_join(permissions::table),
                            ),
                        ),
                    )
                    .select((
                        permissions::columns::name.nullable(),
                        roles::columns::is_admin,
                    ))
                    .distinct()
                    // .filter(users::columns::id.eq(user_id))
                    .get_results::<(Option<String>, bool)>(conn)?;

                // For permissions, just collect em all
                // For is_admin, do an or-map (true if any are true)
                let (permissions, is_admin) = permissions_query_result
                    .into_iter()
                    .try_fold::<_, _, ResponseResult<_>>(
                    (HashSet::new(), false),
                    |(mut permissions, is_admin),
                     (permission_opt, is_admin_row)| {
                        if let Some(permission) = permission_opt {
                            // Parsing here shouldn't ever fail because
                            // PermissionType covers all DB values
                            permissions.insert(permission.parse()?);
                        }
                        Ok((permissions, is_admin || is_admin_row))
                    },
                )?;

                Some(Self {
                    user_provider_id,
                    user: Some(AuthorizedUser {
                        id: user_id,
                        username,
                        permissions,
                        is_admin,
                    }),
                })
            }
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
        let user_context = match user_provider_id {
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
    use crate::util;
    use maplit::hashset;

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
        let user_provider: models::UserProvider = models::NewUserProvider {
            sub: "sub",
            provider_name: "provider",
            user_id: None,
        }
        .insert()
        .returning(user_providers::all_columns)
        .get_result(conn)
        .unwrap();

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

        let user: models::User = models::NewUser { username: "user1" }
            .insert()
            .returning(users::all_columns)
            .get_result(conn)
            .unwrap();
        let user_provider: models::UserProvider = models::NewUserProvider {
            sub: "sub",
            provider_name: "provider",
            user_id: Some(user.id),
        }
        .insert()
        .returning(user_providers::all_columns)
        .get_result(conn)
        .unwrap();
        user.add_roles_x(conn, &[models::RoleType::SpecCreator])
            .unwrap();

        let ctx = UserContext::load_context(conn, user_provider.id);
        assert_eq!(
            ctx.unwrap(),
            Some(UserContext {
                user_provider_id: user_provider.id,
                user: Some(AuthorizedUser {
                    id: user.id,
                    username: user.username,
                    permissions: hashset! {models::PermissionType::CreateSpecs},
                    is_admin: false
                })
            })
        );
    }

    #[test]
    fn test_user_load_context_initialized_user_admin() {
        let pool = util::init_test_db_conn_pool().unwrap();
        let conn = &pool.get().unwrap();

        let user: models::User = models::NewUser { username: "user1" }
            .insert()
            .returning(users::all_columns)
            .get_result(conn)
            .unwrap();
        let user_provider: models::UserProvider = models::NewUserProvider {
            sub: "sub",
            provider_name: "provider",
            user_id: Some(user.id),
        }
        .insert()
        .returning(user_providers::all_columns)
        .get_result(conn)
        .unwrap();
        user.add_roles_x(
            conn,
            &[models::RoleType::SpecCreator, models::RoleType::Admin],
        )
        .unwrap();

        let ctx = UserContext::load_context(conn, user_provider.id);
        assert_eq!(
            ctx.unwrap(),
            Some(UserContext {
                user_provider_id: user_provider.id,
                user: Some(AuthorizedUser {
                    id: user.id,
                    username: user.username,
                    permissions: hashset! {models::PermissionType::CreateSpecs},
                    is_admin: true
                })
            })
        );
    }
}
