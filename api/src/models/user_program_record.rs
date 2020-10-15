use std::cmp;

use crate::{error::ResponseResult, schema::user_program_records};
use chrono::{DateTime, Utc};
use diesel::{
    dsl, expression::bound::Bound, query_builder::InsertStatement, sql_types,
    ExpressionMethods, Identifiable, Insertable, PgConnection, QueryDsl,
    Queryable, RunQueryDsl,
};
use uuid::Uuid;

/// Expression to filter users by username
type WithUserAndProgramSpec = dsl::And<
    dsl::Eq<
        user_program_records::columns::user_id,
        Bound<sql_types::Uuid, Uuid>,
    >,
    dsl::Eq<
        user_program_records::columns::program_spec_id,
        Bound<sql_types::Uuid, Uuid>,
    >,
>;

/// A record of a **successful** program execution. The program must compile,
/// execute, and have valid output (matching the program spec) for this a row
/// in this table to be valid. This model does not identify which user executed
/// the program or which program spec it was written for. This is just meant to
/// be linked to by `user_programs` or `user_program_pbs` to store stats data
/// for a program.
#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "user_program_records"]
pub struct UserProgramRecord {
    /// DB primary key
    pub id: Uuid,
    /// The user that executed this program
    pub user_id: Uuid,
    /// The spec for the program being executed
    pub program_spec_id: Uuid,
    /// GDLK code that was executed
    pub source_code: String,
    /// Number of CPU cycles required for the program to terminate
    pub cpu_cycles: i32,
    /// Number of compiled instructions in the program (i.e. program size)
    pub instructions: i32,
    /// Number of unique user registers referenced by the program
    pub registers_used: i32,
    /// Number of unique stacks referenced by the program
    pub stacks_used: i32,
    /// When this row was created, which is also when the program was executed.
    pub created: DateTime<Utc>,
}

/// A set of statistics gathered on an executed user_program. This could be a
/// minimized version of a `user_program_records` row, or it could be an
/// aggregated version based on multiple rows.
#[derive(Copy, Clone, Debug, PartialEq, Queryable)]
pub struct UserProgramRecordStats {
    /// Number of CPU cycles required for the program to terminate
    pub cpu_cycles: i32,
    /// Number of compiled instructions in the program (i.e. program size)
    pub instructions: i32,
    /// Number of unique user registers referenced by the program
    pub registers_used: i32,
    /// Number of unique stacks referenced by the program
    pub stacks_used: i32,
}

impl UserProgramRecord {
    /// Filter the `user_program_records` table for a particular
    /// user+program_spec.
    pub fn filter_by_user_and_program_spec(
        user_id: Uuid,
        program_spec_id: Uuid,
    ) -> dsl::Filter<user_program_records::table, WithUserAndProgramSpec> {
        user_program_records::table
            .filter(user_program_records::columns::user_id.eq(user_id))
            .filter(
                user_program_records::columns::program_spec_id
                    .eq(program_spec_id),
            )
    }

    /// Load all stat Personal Bests for a user+program_spec. If the user has
    /// never executed a solution for that program_spec, the result will be
    /// `Ok(None)`. Otherwise, return the best value for each stat. The stats
    /// may not all be from the same solution; stats are aggregated to get the
    /// best stat across all solutions.
    pub fn load_pbs_x(
        conn: &PgConnection,
        user_id: Uuid,
        program_spec_id: Uuid,
    ) -> ResponseResult<Option<UserProgramRecordStats>> {
        // TODO switch to a GROUP BY after https://github.com/diesel-rs/diesel/issues/210

        // Load all records for this user+program_spec
        let result: Vec<UserProgramRecordStats> =
            Self::filter_by_user_and_program_spec(user_id, program_spec_id)
                .select((
                    user_program_records::columns::cpu_cycles,
                    user_program_records::columns::instructions,
                    user_program_records::columns::registers_used,
                    user_program_records::columns::stacks_used,
                ))
                .get_results(conn)?;

        // For each stat, find the min value (or None if no results)
        let reduced: Option<UserProgramRecordStats> = match result.as_slice() {
            [] => None,
            [first, ref rest @ ..] => {
                Some(rest.iter().fold(*first, |acc, row| {
                    UserProgramRecordStats {
                        cpu_cycles: cmp::min(acc.cpu_cycles, row.cpu_cycles),
                        instructions: cmp::min(
                            acc.instructions,
                            row.instructions,
                        ),
                        registers_used: cmp::min(
                            acc.registers_used,
                            row.registers_used,
                        ),
                        stacks_used: cmp::min(acc.stacks_used, row.stacks_used),
                    }
                }))
            }
        };
        Ok(reduced)
    }
}

#[derive(Clone, Debug, Insertable)]
#[table_name = "user_program_records"]
pub struct NewUserProgramRecord<'a> {
    pub user_id: Uuid,
    pub program_spec_id: Uuid,
    pub source_code: &'a str,
    pub cpu_cycles: i32,
    pub instructions: i32,
    pub registers_used: i32,
    pub stacks_used: i32,
}

impl NewUserProgramRecord<'_> {
    /// Insert this object into the `user_program_records` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<
        user_program_records::table,
        <Self as Insertable<user_program_records::table>>::Values,
    > {
        self.insert_into(user_program_records::table)
    }
}
