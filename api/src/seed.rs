//! Development-only module used to seed data into the DB for testing and
//! development.

use crate::{
    models::{NewHardwareSpec, NewProgramSpec, NewUser, NewUserProgram},
    schema::{hardware_specs, program_specs, users},
};
use diesel::{PgConnection, RunQueryDsl};
use uuid::Uuid;

/// Inserts hard-coded seed data into the DB. Related objects use dynamic FKs
/// rather than hard-coded ones, to prevent weird situations with existing data.
///
/// This implementation is pretty primitive right now. We'll probably want to
/// expand it as we add more models.
pub fn seed_db(conn: &PgConnection) -> Result<(), diesel::result::Error> {
    let user_id = NewUser { username: "user1" }
        .insert()
        .returning(users::id)
        .get_result(conn)?;

    let hardware_spec_id: Uuid = NewHardwareSpec {
        slug: "hw1",
        num_registers: 1,
        num_stacks: 0,
        max_stack_length: 0,
    }
    .insert()
    .returning(hardware_specs::dsl::id)
    .get_result(conn)?;

    let prog1_spec_id = NewProgramSpec {
        slug: "prog1",
        hardware_spec_id,
        input: vec![1, 2, 3],
        expected_output: vec![1, 2, 3],
    }
    .insert()
    .returning(program_specs::id)
    .get_result(conn)?;

    NewProgramSpec {
        slug: "prog2",
        hardware_spec_id,
        input: vec![1, 2, 3],
        expected_output: vec![2, 4, 6],
    }
    .insert()
    .execute(conn)?;

    // Source code for one program
    NewUserProgram {
        user_id,
        program_spec_id: prog1_spec_id,
        file_name: "program.gdlk",
        source_code: "READ\nWRITE\nREAD\nWRITE\nREAD\nWRITE\n",
    }
    .insert()
    .execute(conn)
    .unwrap();

    Ok(())
}
