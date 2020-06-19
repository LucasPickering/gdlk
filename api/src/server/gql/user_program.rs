use crate::{
    error::ResponseResult,
    models,
    schema::user_programs,
    server::gql::{
        internal::{GenericEdge, NodeType},
        ConnectionPageParams, Context, CreateUserProgramPayloadFields, Cursor,
        DeleteUserProgramPayloadFields, PageInfo, ProgramSpecNode,
        UpdateUserProgramPayloadFields, UserNode, UserProgramConnectionFields,
        UserProgramEdgeFields, UserProgramNodeFields,
    },
    util::{self, Valid},
};
use chrono::{offset::Utc, DateTime};
use diesel::{dsl, PgConnection, QueryDsl, QueryResult, RunQueryDsl, Table};
use juniper::ID;
use juniper_from_schema::{QueryTrail, Walked};
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct UserProgramNode {
    pub user_program: models::UserProgram,
}

impl From<models::UserProgram> for UserProgramNode {
    fn from(model: models::UserProgram) -> Self {
        Self {
            user_program: model,
        }
    }
}

impl NodeType for UserProgramNode {
    type Model = models::UserProgram;

    fn find(conn: &PgConnection, id: Uuid) -> QueryResult<Self::Model> {
        user_programs::table.find(id).get_result(conn)
    }
}

impl UserProgramNodeFields for UserProgramNode {
    fn field_id(&self, _executor: &juniper::Executor<'_, Context>) -> ID {
        util::uuid_to_gql_id(self.user_program.id)
    }

    fn field_file_name(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &String {
        &self.user_program.file_name
    }

    fn field_source_code(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &String {
        &self.user_program.source_code
    }

    fn field_created(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &DateTime<Utc> {
        &self.user_program.created
    }

    fn field_last_modified(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &DateTime<Utc> {
        &self.user_program.last_modified
    }

    fn field_user(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserNode, Walked>,
    ) -> ResponseResult<UserNode> {
        Ok(UserNode::find(
            &executor.context().get_db_conn()? as &PgConnection,
            self.user_program.user_id,
        )?
        .into())
    }

    fn field_program_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, ProgramSpecNode, Walked>,
    ) -> ResponseResult<ProgramSpecNode> {
        Ok(ProgramSpecNode::find(
            &executor.context().get_db_conn()? as &PgConnection,
            self.user_program.program_spec_id,
        )?
        .into())
    }
}

pub type UserProgramEdge = GenericEdge<UserProgramNode>;

impl UserProgramEdgeFields for UserProgramEdge {
    fn field_node(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserProgramNode, Walked>,
    ) -> &UserProgramNode {
        self.node()
    }

    fn field_cursor(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &Cursor {
        self.cursor()
    }
}

/// "Connection" is a concept from Relay. Read more: https://graphql.org/learn/pagination/
pub struct UserProgramConnection {
    user_id: Uuid,
    program_spec_id: Uuid,
    page_params: Valid<ConnectionPageParams>,
}

impl UserProgramConnection {
    pub fn new(
        user_id: Uuid,
        program_spec_id: Uuid,
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> ResponseResult<Self> {
        Ok(Self {
            user_id,
            program_spec_id,
            page_params: ConnectionPageParams::new(first, after)?,
        })
    }

    fn get_total_count(&self, context: &Context) -> ResponseResult<i32> {
        match models::UserProgram::filter_by_user_and_program_spec(
            self.user_id,
            self.program_spec_id,
        )
        .select(dsl::count_star())
        .get_result::<i64>(&context.get_db_conn()?)
        {
            // Convert i64 to i32 - if this fails, we're in a rough spot
            Ok(count) => Ok(count.try_into().unwrap()),
            Err(err) => Err(err.into()),
        }
    }

    fn get_edges(
        &self,
        context: &Context,
    ) -> ResponseResult<Vec<UserProgramEdge>> {
        let offset = self.page_params.offset();

        // Load data from the query
        let mut query = models::UserProgram::filter_by_user_and_program_spec(
            self.user_id,
            self.program_spec_id,
        )
        .select(user_programs::table::all_columns())
        .offset(offset.into())
        .into_boxed();

        // Conditionally include limit param
        if let Some(limit) = self.page_params.limit() {
            query = query.limit(limit.into());
        }

        let rows: Vec<models::UserProgram> =
            query.get_results(&context.get_db_conn()?)?;

        Ok(UserProgramEdge::from_db_rows(rows, offset))
    }
}

impl UserProgramConnectionFields for UserProgramConnection {
    fn field_total_count(
        &self,
        executor: &juniper::Executor<'_, Context>,
    ) -> ResponseResult<i32> {
        self.get_total_count(executor.context())
    }

    fn field_page_info(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, PageInfo, Walked>,
    ) -> ResponseResult<PageInfo> {
        Ok(PageInfo::from_page_params(
            &self.page_params,
            self.get_total_count(executor.context())?,
        ))
    }

    fn field_edges(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserProgramEdge, Walked>,
    ) -> ResponseResult<Vec<UserProgramEdge>> {
        self.get_edges(executor.context())
    }
}

pub struct CreateUserProgramPayload {
    pub user_program: models::UserProgram,
}

impl CreateUserProgramPayloadFields for CreateUserProgramPayload {
    fn field_user_program_edge(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserProgramEdge, Walked>,
    ) -> UserProgramEdge {
        GenericEdge::from_db_row(self.user_program.clone(), 0)
    }
}

pub struct UpdateUserProgramPayload {
    pub user_program: Option<models::UserProgram>,
}

impl UpdateUserProgramPayloadFields for UpdateUserProgramPayload {
    fn field_user_program_edge(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserProgramEdge, Walked>,
    ) -> Option<UserProgramEdge> {
        self.user_program
            .as_ref()
            // Since this wasn't queried as part of a set, we can use a
            // bullshit offset to generate the cursor
            .map(|row| GenericEdge::from_db_row(row.clone(), 0))
    }
}

pub struct DeleteUserProgramPayload {
    pub deleted_id: Option<Uuid>,
}

impl DeleteUserProgramPayloadFields for DeleteUserProgramPayload {
    fn field_deleted_id(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> Option<juniper::ID> {
        self.deleted_id.map(util::uuid_to_gql_id)
    }
}
