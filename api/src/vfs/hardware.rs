//! Handlers for files specific to a hardware spec. Structure looks like:
//!
//! ```
//! <slug>/
//!   spec.txt
//!   programs/
//!     <self::program>
//! ```

use crate::{
    error::Result,
    models::HardwareSpec,
    schema::hardware_specs,
    vfs::{
        internal::{Context, PathVariables, VirtualNodeHandler},
        NodePermissions, PERMS_R, PERMS_RX,
    },
};
use diesel::{
    dsl::{exists, select},
    QueryDsl, RunQueryDsl,
};

/// Serves all hardware spec directories.
#[derive(Debug)]
pub struct HardwareSpecNodeHandler();

impl VirtualNodeHandler for HardwareSpecNodeHandler {
    fn exists(
        &self,
        context: &Context,
        _: &PathVariables,
        hw_spec_slug: &str,
    ) -> Result<bool> {
        Ok(select(exists(HardwareSpec::filter_by_slug(hw_spec_slug)))
            .get_result(context.conn())?)
    }

    fn permissions(
        &self,
        _: &Context,
        _: &PathVariables,
        _: &str,
    ) -> Result<NodePermissions> {
        Ok(PERMS_RX)
    }

    fn list_variable_nodes(
        &self,
        context: &Context,
        _: &PathVariables,
    ) -> Result<Vec<String>> {
        let hw_spec_slugs: Vec<String> = hardware_specs::dsl::hardware_specs
            .select(hardware_specs::dsl::slug)
            .get_results(context.conn())?;
        Ok(hw_spec_slugs)
    }
}

/// Serves the `spec.txt` file for a particular hardware spec.
#[derive(Debug)]
pub struct HardwareSpecFileNodeHandler();

impl VirtualNodeHandler for HardwareSpecFileNodeHandler {
    fn permissions(
        &self,
        _: &Context,
        _: &PathVariables,
        _: &str,
    ) -> Result<NodePermissions> {
        Ok(PERMS_R)
    }

    fn content(
        &self,
        context: &Context,
        path_variables: &PathVariables,
        _: &str,
    ) -> Result<String> {
        let hw_spec_slug = path_variables.get_var("hw_spec_slug");
        let hw_spec: HardwareSpec = HardwareSpec::filter_by_slug(hw_spec_slug)
            .get_result(context.conn())?;
        Ok(format!(
            "Registers: {}\nStacks: {}\nMax stack length: {}\n",
            hw_spec.num_registers, hw_spec.num_stacks, hw_spec.max_stack_length
        ))
    }
}
