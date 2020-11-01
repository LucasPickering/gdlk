use crate::{
    error::{ApiResult, ClientError, DbErrorConverter},
    models,
    schema::hardware_specs,
    views::{RequestContext, View},
};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl, Table};
use uuid::Uuid;
use validator::Validate;

/// Create a new hardware spec
pub struct CreateHardwareSpecView<'a> {
    pub context: &'a RequestContext,
    pub name: &'a str,
    pub num_registers: i32,
    pub num_stacks: i32,
    pub max_stack_length: i32,
}

impl<'a> View for CreateHardwareSpecView<'a> {
    type Output = models::HardwareSpec;

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
        let new_hardware_spec = models::NewHardwareSpec {
            name: self.name,
            num_registers: self.num_registers,
            num_stacks: self.num_stacks,
            max_stack_length: self.max_stack_length,
        };
        new_hardware_spec.validate()?;

        // Insert the new row and return the whole row
        let result: Result<models::HardwareSpec, _> = new_hardware_spec
            .insert()
            .returning(hardware_specs::table::all_columns())
            .get_result(&self.context.db_conn()?);

        DbErrorConverter {
            // HardwareSpec already exists with this name or slug
            unique_violation_to_exists: true,
            ..Default::default()
        }
        .convert(result)
    }
}

/// Modify an existing hardware spec
pub struct UpdateHardwareSpecView<'a> {
    pub context: &'a RequestContext,
    pub id: Uuid,
    pub name: Option<&'a str>,
    pub num_registers: Option<i32>,
    pub num_stacks: Option<i32>,
    pub max_stack_length: Option<i32>,
}

impl<'a> View for UpdateHardwareSpecView<'a> {
    type Output = Option<models::HardwareSpec>;

    fn check_permissions(&self) -> ApiResult<()> {
        let user = self.context.user()?;
        if user.has_permission(models::PermissionType::ModifyAllSpecs) {
            Ok(())
        } else {
            Err(ClientError::PermissionDenied.into())
        }
    }

    fn execute_internal(&self) -> ApiResult<Self::Output> {
        // User a helper type to do the insert
        let modified_hardware_spec = models::ModifiedHardwareSpec {
            id: self.id,
            name: self.name,
            num_registers: self.num_registers,
            num_stacks: self.num_stacks,
            max_stack_length: self.max_stack_length,
        };
        modified_hardware_spec.validate()?;

        // Update the row, returning the new value. If the row doesn't exist,
        // this will return None.
        let result: Result<Option<models::HardwareSpec>, _> =
            diesel::update(hardware_specs::table.find(self.id))
                .set(modified_hardware_spec)
                .returning(hardware_specs::table::all_columns())
                .get_result(&self.context.db_conn()?)
                .optional();

        DbErrorConverter {
            // HardwareSpec already exists with this name or slug
            unique_violation_to_exists: true,
            // No update fields were given
            query_builder_to_no_update: true,
            ..Default::default()
        }
        .convert(result)
    }
}
