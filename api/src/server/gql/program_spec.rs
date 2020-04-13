use crate::{
    error::ServerResult,
    models,
    schema::{program_specs, user_programs},
    server::gql::{
        hardware_spec::HardwareSpecNode,
        internal::{GenericEdge, NodeType},
        user_program::{UserProgramConnection, UserProgramNode},
        ConnectionPageParams, Context, Cursor, PageInfo,
        ProgramSpecConnectionFields, ProgramSpecEdgeFields,
        ProgramSpecNodeFields,
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
        util::uuid_to_gql_id(self.program_spec.id)
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
        user_id: juniper::ID,
        file_name: String,
    ) -> ServerResult<Option<UserProgramNode>> {
        Ok(models::UserProgram::filter_by_user_and_program_spec(
            util::gql_id_to_uuid(&user_id)?,
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
        user_id: juniper::ID,
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> ServerResult<UserProgramConnection> {
        UserProgramConnection::new(
            util::gql_id_to_uuid(&user_id)?,
            self.program_spec.id,
            first,
            after,
        )
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
            .select(dsl::count_star())
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
