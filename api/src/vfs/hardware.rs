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
    models::FullHardwareSpec,
    schema::hardware_specs,
    vfs::{
        internal::{Context, VirtualNodeHandler},
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
    fn exists(&self, context: &Context, hw_spec_slug: &str) -> Result<bool> {
        Ok(
            select(exists(FullHardwareSpec::filter_by_slug(hw_spec_slug)))
                .get_result(context.db_conn)?,
        )
    }

    fn get_permissions(&self, _: &Context, _: &str) -> Result<NodePermissions> {
        Ok(PERMS_RX)
    }

    fn list_physical_nodes(&self, context: &Context) -> Result<Vec<String>> {
        let hw_spec_slugs: Vec<String> = hardware_specs::dsl::hardware_specs
            .select(hardware_specs::dsl::slug)
            .get_results(context.db_conn)?;
        Ok(hw_spec_slugs)
    }
}

/// Serves the `spec.txt` file for a particular hardware spec.
#[derive(Debug)]
pub struct HardwareSpecFileNodeHandler();

impl VirtualNodeHandler for HardwareSpecFileNodeHandler {
    fn get_permissions(
        &self,
        _context: &Context,
        _path_segment: &str,
    ) -> Result<NodePermissions> {
        Ok(PERMS_R)
    }

    fn get_content(&self, context: &Context, _: &str) -> Result<String> {
        let hw_spec_slug = context.get_var("hw_spec_slug");
        let hw_spec: FullHardwareSpec =
            FullHardwareSpec::filter_by_slug(hw_spec_slug)
                .get_result(context.db_conn)?;
        Ok(format!(
            "Registers: {}\nStacks: {}\nMax stack length: {}\n",
            hw_spec.num_registers, hw_spec.num_stacks, hw_spec.max_stack_length
        ))
    }
}
