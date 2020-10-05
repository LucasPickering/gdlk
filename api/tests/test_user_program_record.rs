#![deny(clippy::all)]

use std::collections::HashSet;

use crate::utils::{factories::*, ContextBuilder};
use diesel::{dsl, QueryDsl, RunQueryDsl};
use diesel_factories::Factory;
use gdlk_api::{models, schema::user_program_records};
use maplit::hashset;
use uuid::Uuid;

mod utils;

/// Test the `delete_dangling_x` function, which deletes all
/// user_program_records that are no longer referenced by any other rows.
#[test]
fn test_delete_dangling() {
    let mut context_builder = ContextBuilder::new();
    let user = context_builder.log_in(&[]);
    let conn = context_builder.db_conn();

    UserProgramRecordFactory::default().insert(conn);
    let solution_record = UserProgramRecordFactory::default().insert(conn);
    let pb_record = UserProgramRecordFactory::default().insert(conn);

    let hw_spec = HardwareSpecFactory::default().name("HW 1").insert(conn);
    let program_spec = ProgramSpecFactory::default()
        .name("prog1")
        .hardware_spec(&hw_spec)
        .input(vec![1])
        .expected_output(vec![1])
        .insert(conn);

    UserProgramFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .file_name("solution.gdlk")
        .record(Some(&solution_record))
        .insert(conn);
    UserProgramPbFactory::default()
        .user(&user)
        .program_spec(&program_spec)
        .record(&pb_record)
        .stat(models::StatType::CpuCycles.to_str())
        .insert(conn);

    assert_eq!(
        user_program_records::table
            .select(dsl::count(user_program_records::columns::id))
            .get_result::<i64>(conn)
            .unwrap(),
        3
    );

    models::UserProgramRecord::delete_dangling_x(conn).unwrap();
    let remaining_ids: HashSet<Uuid> = user_program_records::table
        .select(user_program_records::columns::id)
        .get_results::<Uuid>(conn)
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(remaining_ids, hashset! {solution_record.id, pb_record.id});
}
