#![deny(clippy::all)]

use crate::utils::factories::*;
use diesel_factories::Factory;
use gdlk_api::{models, util};
use uuid::Uuid;

mod utils;

/// Test the `load_pbs_x` function, loads all stat PBs for a user+program_spec
#[actix_rt::test]
async fn test_load_pbs_x() {
    let conn = util::init_test_db_conn_pool().unwrap().get().unwrap();

    let user = UserFactory::default().username("user1").insert(&conn);
    let hw_spec = HardwareSpecFactory::default().name("HW1").insert(&conn);
    let prog_spec = ProgramSpecFactory::default()
        .hardware_spec(&hw_spec)
        .name("Prog1")
        .insert(&conn);
    // Create some extra rows to make sure we don't pull in extra rows
    let other_user =
        UserFactory::default().username("other user").insert(&conn);
    let other_prog_spec = ProgramSpecFactory::default()
        .hardware_spec(&hw_spec)
        .name("Prog2")
        .insert(&conn);

    // user+prog_spec
    UserProgramRecordFactory::default()
        .user(&user)
        .program_spec(&prog_spec)
        // These 2 are PBs
        .cpu_cycles(1)
        .instructions(2)
        // These 2 are NOT PBs
        .registers_used(10)
        .stacks_used(10)
        .insert(&conn);
    UserProgramRecordFactory::default()
        .user(&user)
        .program_spec(&prog_spec)
        // These 2 are NOT PBs
        .cpu_cycles(10)
        .instructions(10)
        // These 2 are PBs
        .registers_used(3)
        .stacks_used(4)
        .insert(&conn);

    // These 2 are decoys to make sure the query is filtering properly
    UserProgramRecordFactory::default()
        .user(&other_user)
        .program_spec(&prog_spec)
        .insert(&conn);
    UserProgramRecordFactory::default()
        .user(&user)
        .program_spec(&other_prog_spec)
        .insert(&conn);

    assert_eq!(
        models::UserProgramRecord::load_pbs_x(&conn, user.id, prog_spec.id)
            .unwrap(),
        Some(models::UserProgramRecordStats {
            cpu_cycles: 1,
            instructions: 2,
            registers_used: 3,
            stacks_used: 4,
        })
    );

    // Make an empty query, make sure it gives no results
    assert_eq!(
        models::UserProgramRecord::load_pbs_x(&conn, user.id, Uuid::nil())
            .unwrap(),
        None
    )
}
