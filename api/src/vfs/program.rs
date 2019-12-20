//! Handlers for files specific to a hardware spec/program spec combo. Structure
//! looks like:
//!
//! ```
//! <program_spec_slug>/
//!   spec.txt
//!   program.gdlk
//! ```

use crate::{
    error::Result,
    models::FullProgramSpec,
    schema::program_specs,
    vfs::{
        Context, NodePermissions, VirtualNodeHandler, PERMS_R, PERMS_RW,
        PERMS_RX,
    },
};
use diesel::{
    dsl::{exists, select},
    QueryDsl, RunQueryDsl,
};
use gdlk::ast::LangValue;

/// Serves all program spec directories.
#[derive(Debug)]
pub struct ProgramSpecNodeHandler();

impl VirtualNodeHandler for ProgramSpecNodeHandler {
    fn exists(
        &self,
        context: &Context,
        program_spec_slug: &str,
    ) -> Result<bool> {
        let hw_spec_slug = context.get_var("hw_spec_slug");
        Ok(select(exists(FullProgramSpec::filter_by_slugs(
            hw_spec_slug,
            program_spec_slug,
        )))
        .get_result(context.db_conn)?)
    }

    fn get_permissions(&self, _: &Context, _: &str) -> Result<NodePermissions> {
        Ok(PERMS_RX)
    }

    fn list_physical_nodes(&self, context: &Context) -> Result<Vec<String>> {
        let hw_spec_slug = context.get_var("hw_spec_slug");
        let program_spec_slugs: Vec<String> =
            FullProgramSpec::filter_by_hw_slug(hw_spec_slug)
                .select(program_specs::dsl::slug)
                .get_results(context.db_conn)?;
        Ok(program_spec_slugs)
    }
}

/// Serves the `spec.txt` file for a program spec.
#[derive(Debug)]
pub struct ProgramSpecFileNodeHandler();

impl VirtualNodeHandler for ProgramSpecFileNodeHandler {
    fn get_permissions(
        &self,
        _context: &Context,
        _path_segment: &str,
    ) -> Result<NodePermissions> {
        Ok(PERMS_R)
    }

    fn get_content(&self, context: &Context, _: &str) -> Result<String> {
        let hw_spec_slug = context.get_var("hw_spec_slug");
        let program_spec_slug = context.get_var("program_spec_slug");
        let (input, expected_output): (Vec<LangValue>, Vec<LangValue>) =
            FullProgramSpec::filter_by_slugs(hw_spec_slug, program_spec_slug)
                .select((
                    program_specs::dsl::input,
                    program_specs::dsl::expected_output,
                ))
                .get_result(context.db_conn)?;
        Ok(format!(
            "Input: {:?}\nExpected output: {:?}\n",
            input, expected_output
        ))
    }
}

/// Serves the `program.gdlk` file for a program spec. At some point this will
/// probably become variable, so that a user can have multiple source files for
/// one program spec.
#[derive(Debug)]
pub struct ProgramSourceNodeHandler();

impl VirtualNodeHandler for ProgramSourceNodeHandler {
    fn get_permissions(&self, _: &Context, _: &str) -> Result<NodePermissions> {
        Ok(PERMS_RW)
    }

    fn get_content(&self, _: &Context, _: &str) -> Result<String> {
        Ok("TODO".into())
    }
}
