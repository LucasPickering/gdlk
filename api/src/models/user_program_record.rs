use crate::{
    error::{ResponseError, ResponseResult},
    schema::{user_program_pbs, user_program_records, user_programs},
};
use chrono::{DateTime, Utc};
use diesel::{
    dsl, query_builder::InsertStatement, ExpressionMethods, Identifiable,
    Insertable, NullableExpressionMethods, PgConnection, QueryDsl, Queryable,
    RunQueryDsl,
};
use uuid::Uuid;

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

impl UserProgramRecord {
    /// Delete rows in the `user_program_records` table that are not referenced
    /// from anywhere. This checks both the `user_programs` and
    /// `user_program_pbs` tables. This builds and executes the query. Returns
    /// the number of deleted rows.
    pub fn delete_dangling_x(conn: &PgConnection) -> ResponseResult<usize> {
        diesel::delete(
            user_program_records::table
                .filter(dsl::not(dsl::exists(
                    user_programs::table.filter(
                        user_programs::columns::record_id
                            .eq(user_program_records::columns::id.nullable()),
                    ),
                )))
                .filter(dsl::not(dsl::exists(
                    user_program_pbs::table.filter(
                        user_program_pbs::columns::record_id
                            .eq(user_program_records::columns::id),
                    ),
                ))),
        )
        .execute(conn)
        .map_err(ResponseError::from_server_error)
    }
}

#[derive(Clone, Debug, Insertable)]
#[table_name = "user_program_records"]
pub struct NewUserProgramRecord<'a> {
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
