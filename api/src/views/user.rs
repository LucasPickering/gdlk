use crate::{
    error::{ClientError, DbErrorConverter, ResponseError, ResponseResult},
    models,
    schema::{user_providers, users},
    views::{RequestContext, UserContext, View},
};
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use validator::Validate;

/// Initialize a new user.
pub struct InitializeUserView<'a> {
    pub context: &'a RequestContext,
    pub username: &'a str,
}

impl<'a> View for InitializeUserView<'a> {
    type Output = models::User;

    fn check_permissions(&self) -> ResponseResult<()> {
        // Since we're directly handling the user context in this view, we
        // handle all the not-logged-in logic directly during execution.
        Ok(())
    }

    fn execute_internal(&self) -> ResponseResult<Self::Output> {
        // The user should be logged in, but not had a User object created yet
        match self.context.user_context {
            Some(UserContext {
                user_provider_id, ..
            }) => {
                let conn = self.context.db_conn();
                let new_user = models::NewUser {
                    username: &self.username,
                };
                new_user.validate()?;

                // We need to insert the new user row, then update the
                // user_provider to point at that row. We need a transaction to
                // prevent race conditions.
                conn.transaction::<models::User, ResponseError, _>(|| {
                    let create_user_result: Result<models::User, _> = new_user
                        .insert()
                        .returning(users::all_columns)
                        .get_result(conn);

                    // Check if the username already exists
                    let created_user = DbErrorConverter {
                        unique_violation_to_exists: true,
                        ..Default::default()
                    }
                    .convert(create_user_result)?;

                    // We should update exactly 1 row. If not, then either
                    // the referenced user_provider row is already linked to
                    // a user, or it doesn't exist. In either case, just
                    // return a NotFound error.
                    let updated_rows = diesel::update(
                        user_providers::table
                            .find(user_provider_id)
                            .filter(user_providers::columns::user_id.is_null()),
                    )
                    .set(
                        user_providers::columns::user_id
                            .eq(Some(created_user.id)),
                    )
                    .execute(conn)?;

                    if updated_rows == 0 {
                        Err(ClientError::NotFound { source: None }.into())
                    } else {
                        Ok(created_user)
                    }
                })
            }
            // Get up on outta here
            None => Err(ClientError::Unauthenticated.into()),
        }
    }
}
