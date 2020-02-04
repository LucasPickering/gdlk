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
    models::{ProgramSpec, UserProgram},
    schema::{program_specs, user_programs},
    vfs::{
        internal::PathVariables, Context, NodePermissions, VirtualNodeHandler,
        PERMS_R, PERMS_RW,
    },
};
use diesel::{
    dsl::{exists, select},
    ExpressionMethods, QueryDsl, RunQueryDsl,
};
use gdlk::ast::LangValue;

/// Serves all program spec directories.
#[derive(Debug)]
pub struct ProgramSpecNodeHandler();

impl VirtualNodeHandler for ProgramSpecNodeHandler {
    fn exists(
        &self,
        context: &Context,
        path_variables: &PathVariables,
        program_spec_slug: &str,
    ) -> Result<bool> {
        let hw_spec_slug = path_variables.get_var("hw_spec_slug");
        Ok(select(exists(ProgramSpec::filter_by_slugs(
            hw_spec_slug,
            program_spec_slug,
        )))
        .get_result(context.conn())?)
    }

    fn permissions(
        &self,
        _: &Context,
        _: &PathVariables,
        _: &str,
    ) -> Result<NodePermissions> {
        Ok(PERMS_R)
    }

    fn list_variable_nodes(
        &self,
        context: &Context,
        path_variables: &PathVariables,
    ) -> Result<Vec<String>> {
        let hw_spec_slug = path_variables.get_var("hw_spec_slug");
        let program_spec_slugs: Vec<String> =
            ProgramSpec::filter_by_hw_slug(hw_spec_slug)
                .select(program_specs::dsl::slug)
                .get_results(context.conn())?;
        Ok(program_spec_slugs)
    }
}

/// Serves the `spec.txt` file for a program spec.
#[derive(Debug)]
pub struct ProgramSpecFileNodeHandler();

impl VirtualNodeHandler for ProgramSpecFileNodeHandler {
    fn permissions(
        &self,
        _context: &Context,
        _: &PathVariables,
        _path_segment: &str,
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
        let program_spec_slug = path_variables.get_var("program_spec_slug");
        let (input, expected_output): (Vec<LangValue>, Vec<LangValue>) =
            ProgramSpec::filter_by_slugs(hw_spec_slug, program_spec_slug)
                .select((
                    program_specs::dsl::input,
                    program_specs::dsl::expected_output,
                ))
                .get_result(context.conn())?;
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
    fn exists(
        &self,
        context: &Context,
        path_variables: &PathVariables,
        path_segment: &str,
    ) -> Result<bool> {
        let hw_spec_slug = path_variables.get_var("hw_spec_slug");
        let program_spec_slug = path_variables.get_var("program_spec_slug");

        Ok(select(exists(UserProgram::filter_by_file_name(
            context.user.id,
            hw_spec_slug,
            program_spec_slug,
            path_segment,
        )))
        .get_result(context.conn())?)
    }

    fn permissions(
        &self,
        _: &Context,
        _: &PathVariables,
        _: &str,
    ) -> Result<NodePermissions> {
        Ok(PERMS_RW)
    }

    fn content(
        &self,
        context: &Context,
        path_variables: &PathVariables,
        path_segment: &str,
    ) -> Result<String> {
        let hw_spec_slug = path_variables.get_var("hw_spec_slug");
        let program_spec_slug = path_variables.get_var("program_spec_slug");

        // Get the source stored for this hw/program/user combo
        let source: String = UserProgram::filter_by_file_name(
            context.user.id,
            hw_spec_slug,
            program_spec_slug,
            path_segment,
        )
        .select(user_programs::dsl::source_code)
        .get_result(context.conn())?;
        Ok(source)
    }

    fn list_variable_nodes(
        &self,
        context: &Context,
        path_variables: &PathVariables,
    ) -> Result<Vec<String>> {
        let hw_spec_slug = path_variables.get_var("hw_spec_slug");
        let program_spec_slug = path_variables.get_var("program_spec_slug");

        Ok(UserProgram::filter_by_specs(
            context.user.id,
            hw_spec_slug,
            program_spec_slug,
        )
        .select(user_programs::dsl::file_name)
        .get_results(context.conn())?)
    }

    fn set_content(
        &self,
        context: &Context,
        path_variables: &PathVariables,
        path_segment: &str,
        content: &str,
    ) -> Result<()> {
        use self::user_programs::dsl::*;

        let hw_spec_slug = path_variables.get_var("hw_spec_slug");
        let program_spec_slug = path_variables.get_var("program_spec_slug");

        // Diesel doesn't support updates with joins, so we have to fetch the
        // ID of the UserProgram first, then do the update in a second query.
        let user_program_id: i32 = UserProgram::filter_by_file_name(
            context.user.id,
            hw_spec_slug,
            program_spec_slug,
            path_segment,
        )
        .select(id)
        .get_result(context.conn())?;
        diesel::update(user_programs.filter(id.eq(user_program_id)))
            .set(source_code.eq(content))
            .execute(context.conn())?;

        Ok(())
    }

    fn delete(
        &self,
        context: &Context,
        path_variables: &PathVariables,
        path_segment: &str,
    ) -> Result<()> {
        use self::user_programs::dsl::*;

        let hw_spec_slug = path_variables.get_var("hw_spec_slug");
        let program_spec_slug = path_variables.get_var("program_spec_slug");

        // Diesel doesn't support updates with joins, so we have to fetch the
        // ID of the UserProgram first, then do the update in a second query.
        let user_program_id: i32 = UserProgram::filter_by_file_name(
            context.user.id,
            hw_spec_slug,
            program_spec_slug,
            path_segment,
        )
        .select(id)
        .get_result(context.conn())?;
        diesel::delete(user_programs.filter(id.eq(user_program_id)))
            .execute(context.conn())?;
        Ok(())
    }
}
