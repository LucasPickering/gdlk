use crate::{
    models::{ProgramSpec, User},
    schema::user_programs,
};
use diesel::{
    dsl, expression::bound::Bound, prelude::*, query_builder::InsertStatement,
    sql_types, Identifiable, Queryable,
};
use uuid::Uuid;

/// Expression to filter user_programs by owner's ID and program spec ID
type WithUserAndProgramSpec = dsl::And<
    dsl::Eq<user_programs::columns::user_id, Bound<sql_types::Uuid, Uuid>>,
    dsl::Eq<
        user_programs::columns::program_spec_id,
        Bound<sql_types::Uuid, Uuid>,
    >,
>;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable, Associations)]
#[belongs_to(User, foreign_key = "user_id")]
#[belongs_to(ProgramSpec, foreign_key = "program_spec_id")]
#[table_name = "user_programs"]
pub struct UserProgram {
    pub id: Uuid,
    pub user_id: Uuid,
    pub program_spec_id: Uuid,
    pub file_name: String,
    pub source_code: String,
}

impl UserProgram {
    /// Start a query that filters this table by associated user's ID and by
    /// associated program spec's ID.
    pub fn filter_by_user_and_program_spec(
        user_id: Uuid,
        program_spec_id: Uuid,
    ) -> dsl::Filter<user_programs::table, WithUserAndProgramSpec> {
        user_programs::table
            .filter(user_programs::dsl::user_id.eq(user_id))
            .filter(user_programs::dsl::program_spec_id.eq(program_spec_id))
    }
}

#[derive(Clone, Debug, PartialEq, Insertable)]
#[table_name = "user_programs"]
pub struct NewUserProgram<'a> {
    pub user_id: Uuid,
    pub program_spec_id: Uuid,
    pub file_name: &'a str,
    pub source_code: &'a str,
}

impl NewUserProgram<'_> {
    /// Insert this object into the `user_programs` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<
        user_programs::table,
        <Self as Insertable<user_programs::table>>::Values,
    > {
        self.insert_into(user_programs::table)
    }
}
