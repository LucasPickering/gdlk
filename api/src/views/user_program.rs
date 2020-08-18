use crate::{
    error::{DbErrorConverter, ResponseResult},
    models,
    schema::user_programs,
    views::View,
};
use diesel::{OptionalExtension, PgConnection, RunQueryDsl, Table};
use uuid::Uuid;
use validator::Validate;

/// Create a new user_program
pub struct CreateUserProgramView<'a> {
    pub conn: &'a PgConnection,
    pub user_id: Uuid,
    pub program_spec_id: Uuid,
    pub file_name: &'a str,
    pub source_code: &'a str,
}

impl<'a> View for CreateUserProgramView<'a> {
    type Output = models::UserProgram;

    fn execute(&self) -> ResponseResult<Self::Output> {
        let new_user_program = models::NewUserProgram {
            user_id: self.user_id,
            program_spec_id: self.program_spec_id,
            file_name: self.file_name,
            source_code: self.source_code,
        };
        new_user_program.validate()?;

        // Insert the new row and return the whole row
        let result: Result<models::UserProgram, _> = new_user_program
            .insert()
            .returning(user_programs::table::all_columns())
            .get_result(self.conn);

        DbErrorConverter {
            // Given program spec ID was invalid. Note: this can also indicate
            // an invalid user ID which would be a server-side bug, but fuck it
            fk_violation_to_not_found: true,
            // UserProgram already exists with this program spec/file name
            unique_violation_to_exists: true,
            ..Default::default()
        }
        .convert(result)
    }
}

/// Update an existing user_program
pub struct UpdateUserProgramView<'a> {
    pub conn: &'a PgConnection,
    pub user_id: Uuid,
    pub id: Uuid,
    pub file_name: Option<&'a str>,
    pub source_code: Option<&'a str>,
}

impl<'a> View for UpdateUserProgramView<'a> {
    type Output = Option<models::UserProgram>;

    fn execute(&self) -> ResponseResult<Self::Output> {
        let modified_user_program = models::ModifiedUserProgram {
            id: self.id,
            file_name: self.file_name,
            source_code: self.source_code,
        };
        modified_user_program.validate()?;

        // Update the row, returning the new value. If the row doesn't
        // exist, this will return None.
        let result: Result<Option<models::UserProgram>, _> =
            diesel::update(models::UserProgram::find_for_user(
                modified_user_program.id,
                self.user_id,
            ))
            .set(modified_user_program)
            .returning(user_programs::table::all_columns())
            .get_result(self.conn)
            .optional();

        DbErrorConverter {
            // UserProgram already exists with this program spec/file name
            unique_violation_to_exists: true,
            // No update fields were given
            query_builder_to_no_update: true,
            ..Default::default()
        }
        .convert(result)
    }
}

/// Duplicate an existing user_program
pub struct CopyUserProgramView<'a> {
    pub conn: &'a PgConnection,
    pub user_id: Uuid,
    pub id: Uuid,
}

impl<'a> View for CopyUserProgramView<'a> {
    type Output = Option<models::UserProgram>;

    fn execute(&self) -> ResponseResult<Self::Output> {
        // The user has to own the user program to copy it
        let existing_user_program: Option<models::UserProgram> =
            models::UserProgram::find_for_user(self.id, self.user_id)
                .get_result(self.conn)
                .optional()?;

        // If the requested user_program exists (for the given user), copy it
        Ok(match existing_user_program {
            None => None,
            Some(user_program) => {
                Some(
                    models::NewUserProgram {
                        user_id: self.user_id,
                        program_spec_id: user_program.program_spec_id,
                        // Generate a new file name
                        file_name: &format!("{} copy", &user_program.file_name),
                        source_code: &user_program.source_code,
                    }
                    .insert()
                    .returning(user_programs::all_columns)
                    .get_result(self.conn)?,
                )
            }
        })
    }
}

/// Delete an existing user_program
pub struct DeleteUserProgramView<'a> {
    pub conn: &'a PgConnection,
    pub user_id: Uuid,
    pub id: Uuid,
}

impl<'a> View for DeleteUserProgramView<'a> {
    type Output = Option<Uuid>;

    fn execute(&self) -> ResponseResult<Self::Output> {
        // User has to own the program to delete it
        Ok(diesel::delete(models::UserProgram::find_for_user(
            self.id,
            self.user_id,
        ))
        .returning(user_programs::columns::id)
        .get_result(self.conn)
        .optional()?)
    }
}
