use crate::{
    error::{ApiResult, ClientError, DbErrorConverter},
    models,
    schema::program_specs,
    views::{RequestContext, View},
};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl, Table};
use uuid::Uuid;
use validator::Validate;

/// Create a new program spec
pub struct CreateProgramSpecView<'a> {
    pub context: &'a RequestContext,
    pub hardware_spec_id: Uuid,
    pub name: &'a str,
    pub description: &'a str,
    pub input: &'a [i32],
    pub expected_output: &'a [i32],
}

impl<'a> View for CreateProgramSpecView<'a> {
    type Output = models::ProgramSpec;

    fn check_permissions(&self) -> ApiResult<()> {
        let user = self.context.user()?;
        if user.has_permission(models::PermissionType::CreateSpecs) {
            Ok(())
        } else {
            Err(ClientError::PermissionDenied.into())
        }
    }

    fn execute_internal(&self) -> ApiResult<Self::Output> {
        // User a helper type to do the insert
        let new_program_spec = models::NewProgramSpec {
            hardware_spec_id: self.hardware_spec_id,
            name: self.name,
            description: self.description,
            input: self.input.into(),
            expected_output: self.expected_output.into(),
        };
        new_program_spec.validate()?;

        // Insert the new row and return the whole row
        let result: Result<models::ProgramSpec, _> = new_program_spec
            .insert()
            .returning(program_specs::table::all_columns())
            .get_result(self.context.db_conn());

        DbErrorConverter {
            // Given hardware spec ID was invalid
            fk_violation_to_not_found: true,
            // ProgramSpec already exists with this name or slug
            unique_violation_to_exists: true,
            ..Default::default()
        }
        .convert(result)
    }
}

/// Modify an existing program spec
pub struct UpdateProgramSpecView<'a> {
    pub context: &'a RequestContext,
    pub id: Uuid,
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub input: Option<&'a [i32]>,
    pub expected_output: Option<&'a [i32]>,
}

impl<'a> View for UpdateProgramSpecView<'a> {
    type Output = Option<models::ProgramSpec>;

    fn check_permissions(&self) -> ApiResult<()> {
        let user = self.context.user()?;
        if user.has_permission(models::PermissionType::ModifyAllSpecs) {
            Ok(())
        } else {
            Err(ClientError::PermissionDenied.into())
        }
    }

    fn execute_internal(&self) -> ApiResult<Self::Output> {
        // Use a helper type to do the insert
        let modified_program_spec = models::ModifiedProgramSpec {
            id: self.id,
            name: self.name.as_deref(),
            description: self.description.as_deref(),
            input: self.input.map(Vec::from),
            expected_output: self.expected_output.map(Vec::from),
        };
        modified_program_spec.validate()?;

        // Update the row, returning the new value. If the row doesn't exist,
        // this will return None.
        let result: Result<Option<models::ProgramSpec>, _> =
            diesel::update(program_specs::table.find(modified_program_spec.id))
                .set(modified_program_spec)
                .returning(program_specs::table::all_columns())
                .get_result(self.context.db_conn())
                .optional();

        DbErrorConverter {
            // ProgramSpec already exists with this name or slug
            unique_violation_to_exists: true,
            // No update fields were given
            query_builder_to_no_update: true,
            ..Default::default()
        }
        .convert(result)
    }
}
