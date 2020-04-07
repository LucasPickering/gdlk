use crate::{
    error::ServerResult,
    models,
    schema::{program_specs, user_programs},
    server::gql::{
        hardware::HardwareSpecNode,
        internal::{GenericEdge, NodeType},
        user::UserNode,
        ConnectionPageParams, Context, Cursor, PageInfo,
        ProgramSpecConnectionFields, ProgramSpecEdgeFields,
        ProgramSpecNodeFields, UserProgramConnectionFields,
        UserProgramEdgeFields, UserProgramNodeFields,
    },
    util,
};
use diesel::{
    dsl, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    QueryResult, RunQueryDsl, Table,
};
use gdlk::{ast::LangValue, Valid};
use juniper::ID;
use juniper_from_schema::{QueryTrail, Walked};
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ProgramSpecNode {
    pub program_spec: models::ProgramSpec,
}

impl From<models::ProgramSpec> for ProgramSpecNode {
    fn from(model: models::ProgramSpec) -> Self {
        Self {
            program_spec: model,
        }
    }
}

impl NodeType for ProgramSpecNode {
    type Model = models::ProgramSpec;

    fn find(conn: &PgConnection, id: Uuid) -> QueryResult<Self::Model> {
        program_specs::table.find(id).get_result(conn)
    }
}

impl ProgramSpecNodeFields for ProgramSpecNode {
    fn field_id(&self, _executor: &juniper::Executor<'_, Context>) -> ID {
        util::uuid_to_gql_id(&self.program_spec.id)
    }

    fn field_slug(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &String {
        &self.program_spec.slug
    }

    fn field_input(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &Vec<LangValue> {
        &self.program_spec.input
    }

    fn field_expected_output(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &Vec<LangValue> {
        &self.program_spec.expected_output
    }

    fn field_hardware_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, HardwareSpecNode, Walked>,
    ) -> ServerResult<HardwareSpecNode> {
        Ok(HardwareSpecNode::find(
            &executor.context().get_db_conn()? as &PgConnection,
            self.program_spec.hardware_spec_id,
        )?
        .into())
    }

    fn field_user_program(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserProgramNode, Walked>,
        username: String,
        file_name: String,
    ) -> ServerResult<Option<UserProgramNode>> {
        Ok(models::UserProgram::filter_by_username_and_program_spec(
            &username,
            self.program_spec.id,
        )
        .filter(user_programs::dsl::file_name.eq(&file_name))
        .select(user_programs::table::all_columns())
        .get_result::<models::UserProgram>(&executor.context().get_db_conn()?)
        .optional()?
        .map(UserProgramNode::from))
    }

    fn field_user_programs(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserProgramConnection, Walked>,
        username: String,
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> ServerResult<UserProgramConnection> {
        UserProgramConnection::new(username, self.program_spec.id, first, after)
    }
}

pub type ProgramSpecEdge = GenericEdge<ProgramSpecNode>;

impl ProgramSpecEdgeFields for ProgramSpecEdge {
    fn field_node(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, ProgramSpecNode, Walked>,
    ) -> &ProgramSpecNode {
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
pub struct ProgramSpecConnection {
    hardware_spec_id: Uuid,
    page_params: Valid<ConnectionPageParams>,
}

impl ProgramSpecConnection {
    pub fn new(
        hardware_spec_id: Uuid,
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> ServerResult<Self> {
        Ok(Self {
            hardware_spec_id,
            page_params: ConnectionPageParams::new(first, after)?,
        })
    }

    fn get_total_count(&self, context: &Context) -> ServerResult<i32> {
        match program_specs::table
            .filter(models::ProgramSpec::with_hardware_spec(
                self.hardware_spec_id,
            ))
            .select(dsl::count(dsl::count_star()))
            .get_result::<i64>(&context.get_db_conn()?)
        {
            // Convert i64 to i32 - if this fails, we're in a rough spot
            Ok(count) => Ok(count.try_into()?),
            Err(err) => Err(err.into()),
        }
    }

    fn get_edges(
        &self,
        context: &Context,
    ) -> ServerResult<Vec<ProgramSpecEdge>> {
        let offset = self.page_params.offset();

        // Load data from the query
        let mut query = program_specs::table
            .filter(models::ProgramSpec::with_hardware_spec(
                self.hardware_spec_id,
            ))
            .offset(offset.into())
            .into_boxed();

        // Conditionally include limit param
        if let Some(limit) = self.page_params.limit() {
            query = query.limit(limit.into());
        }

        let rows: Vec<models::ProgramSpec> =
            query.get_results(&context.get_db_conn()?)?;

        Ok(ProgramSpecEdge::from_db_rows(rows, offset))
    }
}

impl ProgramSpecConnectionFields for ProgramSpecConnection {
    fn field_total_count(
        &self,
        executor: &juniper::Executor<'_, Context>,
    ) -> ServerResult<i32> {
        self.get_total_count(executor.context())
    }

    fn field_page_info(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, PageInfo, Walked>,
    ) -> ServerResult<PageInfo> {
        Ok(PageInfo::from_page_params(
            &self.page_params,
            self.get_total_count(executor.context())?,
        ))
    }

    fn field_edges(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, ProgramSpecEdge, Walked>,
    ) -> ServerResult<Vec<ProgramSpecEdge>> {
        self.get_edges(executor.context())
    }
}

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
        util::uuid_to_gql_id(&self.user_program.id)
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

    fn field_user(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserNode, Walked>,
    ) -> ServerResult<UserNode> {
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
    ) -> ServerResult<ProgramSpecNode> {
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
    username: String,
    program_spec_id: Uuid,
    page_params: Valid<ConnectionPageParams>,
}

impl UserProgramConnection {
    pub fn new(
        username: String,
        program_spec_id: Uuid,
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> ServerResult<Self> {
        Ok(Self {
            username,
            program_spec_id,
            page_params: ConnectionPageParams::new(first, after)?,
        })
    }

    fn get_total_count(&self, context: &Context) -> ServerResult<i32> {
        match models::UserProgram::filter_by_username_and_program_spec(
            &self.username,
            self.program_spec_id,
        )
        .select(dsl::count(dsl::count_star()))
        .get_result::<i64>(&context.get_db_conn()?)
        {
            // Convert i64 to i32 - if this fails, we're in a rough spot
            Ok(count) => Ok(count.try_into()?),
            Err(err) => Err(err.into()),
        }
    }

    fn get_edges(
        &self,
        context: &Context,
    ) -> ServerResult<Vec<UserProgramEdge>> {
        let offset = self.page_params.offset();

        // Load data from the query
        let mut query =
            models::UserProgram::filter_by_username_and_program_spec(
                &self.username,
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
    ) -> ServerResult<i32> {
        self.get_total_count(executor.context())
    }

    fn field_page_info(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, PageInfo, Walked>,
    ) -> ServerResult<PageInfo> {
        Ok(PageInfo::from_page_params(
            &self.page_params,
            self.get_total_count(executor.context())?,
        ))
    }

    fn field_edges(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, UserProgramEdge, Walked>,
    ) -> ServerResult<Vec<UserProgramEdge>> {
        self.get_edges(executor.context())
    }
}
