//! Development-only module used to seed data into the DB for testing and
//! development.

use crate::{
    models::{NewHardwareSpec, NewProgramSpec},
    schema::hardware_specs,
};
use diesel::{PgConnection, RunQueryDsl};

/// Inserts hard-coded seed data into the DB. Related objects use dynamic FKs
/// rather than hard-coded ones, to prevent weird situations with existing data.
///
/// This implementation is pretty primitive right now. We'll probably want to
/// expand it as we add more models.
pub fn seed_db(conn: &PgConnection) -> Result<(), diesel::result::Error> {
    let hardware_spec_id: i32 = NewHardwareSpec {
        slug: "hw1".into(),
        num_registers: 1,
        num_stacks: 0,
        max_stack_length: 0,
    }
    .insert()
    .returning(hardware_specs::dsl::id)
    .get_result(conn)?;

    NewProgramSpec {
        slug: "prog1".into(),
        hardware_spec_id,
        input: vec![1, 2, 3],
        expected_output: vec![1, 2, 3],
    }
    .insert()
    .execute(conn)?;

    Ok(())
}
