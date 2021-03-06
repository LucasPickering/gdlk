#![deny(clippy::all)]

use std::collections::HashSet;

use crate::utils::{factories::*, QueryRunner};
use diesel::{dsl, QueryDsl, RunQueryDsl};
use diesel_factories::Factory;
use gdlk_api::{models, schema::user_program_records, util};
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
            cost: 1150
        })
    );

    // Make an empty query, make sure it gives no results
    assert_eq!(
        models::UserProgramRecord::load_pbs_x(&conn, user.id, Uuid::nil())
            .unwrap(),
        None
    )
}

/// Test the `delete_dangling_records` trigger, which deletes all
/// user_program_records that are no longer referenced by any other rows and are
/// not pbs
#[test]
fn test_delete_dangling() {
    let runner = QueryRunner::new();

    runner.run_with_conn(|conn| {
        let user = UserFactory::default().username("user1").insert(&conn);
        let hw_spec = HardwareSpecFactory::default().name("HW1").insert(&conn);
        let prog_spec = ProgramSpecFactory::default()
            .hardware_spec(&hw_spec)
            .name("Prog1")
            .insert(&conn);
        let mut expected_ids: HashSet<Uuid> = HashSet::new();

        // this should be deleted after the last record since it will no longer
        // has any pbs.
        UserProgramRecordFactory::default()
            .user(&user)
            .program_spec(&prog_spec)
            .cpu_cycles(100)
            .instructions(100)
            .registers_used(6)
            .stacks_used(7)
            .insert(&conn);

        // This is also a trash record but it will be referenced so it will stay
        let to_keep = UserProgramRecordFactory::default()
            .user(&user)
            .program_spec(&prog_spec)
            .cpu_cycles(100)
            .instructions(100)
            .registers_used(6)
            .stacks_used(7)
            .insert(&conn);
        expected_ids.insert(to_keep.id);

        UserProgramFactory::default()
            .user(&user)
            .program_spec(&prog_spec)
            .file_name("existing.gdlk")
            .source_code("READ RX0")
            .record(Some(&to_keep))
            .insert(&conn);

        expected_ids.insert(
            UserProgramRecordFactory::default()
                .user(&user)
                .program_spec(&prog_spec)
                // These 2 are PBs
                .cpu_cycles(1)
                .instructions(2)
                // These 2 are NOT PBs
                .registers_used(10)
                .stacks_used(10)
                .insert(&conn)
                .id,
        );

        expected_ids.insert(
            UserProgramRecordFactory::default()
                .user(&user)
                .program_spec(&prog_spec)
                // This is a PB thats the same as the previous one so it will
                // not be deleted
                .cpu_cycles(1)
                // These 3 are NOT PBs
                .instructions(100)
                .registers_used(10)
                .stacks_used(10)
                .insert(&conn)
                .id,
        );

        expected_ids.insert(
            UserProgramRecordFactory::default()
                .user(&user)
                .program_spec(&prog_spec)
                // These 2 are NOT PBs
                .cpu_cycles(10)
                .instructions(10)
                // These 2 are PBs
                .registers_used(3)
                .stacks_used(4)
                .insert(&conn)
                .id,
        );

        // all records are either pbs or referenced by a user program
        assert_eq!(
            user_program_records::table
                .select(dsl::count_star())
                .get_result::<i64>(conn)
                .unwrap(),
            4
        );

        let remaining_ids: HashSet<Uuid> = user_program_records::table
            .select(user_program_records::columns::id)
            .get_results::<Uuid>(conn)
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(remaining_ids, expected_ids);
    });
}
