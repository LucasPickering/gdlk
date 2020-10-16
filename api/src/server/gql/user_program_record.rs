use crate::{
    models,
    schema::user_program_records,
    server::gql::{
        internal::NodeType, RequestContext, UserProgramRecordNodeFields,
    },
    util,
};
use chrono::{offset::Utc, DateTime};
use diesel::{PgConnection, QueryDsl, QueryResult, RunQueryDsl};
use juniper::ID;
use uuid::Uuid;

/// See description in schema.graphql
#[derive(Clone, Debug)]
pub struct UserProgramRecordNode {
    pub user_program_record: models::UserProgramRecord,
}

impl From<models::UserProgramRecord> for UserProgramRecordNode {
    fn from(model: models::UserProgramRecord) -> Self {
        Self {
            user_program_record: model,
        }
    }
}

impl NodeType for UserProgramRecordNode {
    type Model = models::UserProgramRecord;

    fn find(conn: &PgConnection, id: Uuid) -> QueryResult<Self::Model> {
        user_program_records::table.find(id).get_result(conn)
    }
}

impl UserProgramRecordNodeFields for UserProgramRecordNode {
    fn field_id(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> ID {
        util::uuid_to_gql_id(self.user_program_record.id)
    }

    fn field_cpu_cycles(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        self.user_program_record.cpu_cycles
    }

    fn field_instructions(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        self.user_program_record.instructions
    }

    fn field_registers_used(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        self.user_program_record.registers_used
    }

    fn field_stacks_used(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> i32 {
        self.user_program_record.stacks_used
    }

    fn field_created(
        &self,
        _executor: &juniper::Executor<'_, RequestContext>,
    ) -> &DateTime<Utc> {
        &self.user_program_record.created
    }
}
