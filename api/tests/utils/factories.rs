//! Factory structs for generating test data of our models.

use diesel_factories::{Association, Factory};
use gdlk_api::models::{HardwareSpec, ProgramSpec, User, UserProgramRecord};
use uuid::Uuid;

// TODO change all String to &str after https://github.com/davidpdrsn/diesel-factories/issues/21

#[derive(Clone, Default, Factory)]
#[factory(
    model = "gdlk_api::models::User",
    table = "gdlk_api::schema::users",
    id = "Uuid"
)]
pub struct UserFactory {
    pub username: String,
}

#[derive(Clone, Default, Factory)]
#[factory(
    model = "gdlk_api::models::UserProvider",
    table = "gdlk_api::schema::user_providers",
    id = "Uuid"
)]
pub struct UserProviderFactory<'a> {
    pub sub: String,
    pub provider_name: String,
    pub user: Option<Association<'a, User, UserFactory>>,
}

#[derive(Clone, Factory)]
#[factory(
    model = "gdlk_api::models::HardwareSpec",
    table = "gdlk_api::schema::hardware_specs",
    id = "Uuid"
)]
pub struct HardwareSpecFactory {
    pub name: String,
    pub num_registers: i32,
    pub num_stacks: i32,
    pub max_stack_length: i32,
}

impl Default for HardwareSpecFactory {
    fn default() -> Self {
        Self {
            name: "".into(),
            num_registers: 1,
            num_stacks: 0,
            max_stack_length: 0,
        }
    }
}

#[derive(Clone, Default, Factory)]
#[factory(
    model = "gdlk_api::models::ProgramSpec",
    table = "gdlk_api::schema::program_specs",
    id = "Uuid"
)]
pub struct ProgramSpecFactory<'a> {
    pub name: String,
    pub description: String,
    pub hardware_spec: Association<'a, HardwareSpec, HardwareSpecFactory>,
    pub input: Vec<i32>,
    pub expected_output: Vec<i32>,
}

#[derive(Clone, Default, Factory)]
#[factory(
    model = "gdlk_api::models::UserProgram",
    table = "gdlk_api::schema::user_programs",
    id = "Uuid"
)]
pub struct UserProgramFactory<'a> {
    pub user: Association<'a, User, UserFactory>,
    pub program_spec: Association<'a, ProgramSpec, ProgramSpecFactory<'a>>,
    pub record:
        Option<Association<'a, UserProgramRecord, UserProgramRecordFactory>>,
    pub file_name: String,
    pub source_code: String,
}

#[derive(Clone, Default, Factory)]
#[factory(
    model = "gdlk_api::models::UserProgramRecord",
    table = "gdlk_api::schema::user_program_records",
    id = "Uuid"
)]
pub struct UserProgramRecordFactory {
    pub source_code: String,
    pub cpu_cycles: i32,
    pub instructions: i32,
    pub registers_used: i32,
    pub stacks_used: i32,
}

#[derive(Clone, Default, Factory)]
#[factory(
    model = "gdlk_api::models::UserProgramPb",
    table = "gdlk_api::schema::user_program_pbs",
    id = "Uuid"
)]
pub struct UserProgramPbFactory<'a> {
    pub user: Association<'a, User, UserFactory>,
    pub program_spec: Association<'a, ProgramSpec, ProgramSpecFactory<'a>>,
    pub record: Association<'a, UserProgramRecord, UserProgramRecordFactory>,
    pub stat: String,
}
