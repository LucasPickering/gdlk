use crate::{
    error::ServerResult,
    models,
    schema::program_specs,
    server::gql::{
        internal::{GenericEdge, NodeType},
        ConnectionPageParams, Context, Cursor, PageInfo,
        ProgramSpecConnectionFields, ProgramSpecEdgeFields,
        ProgramSpecNodeFields,
    },
    util,
};
use diesel::{
    dsl, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    QueryResult, RunQueryDsl,
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

impl ProgramSpecNode {
    /// Query for a program spec by its parent hardware spec, and by slug.
    pub fn from_slug(
        conn: &PgConnection,
        hardware_spec_id: Uuid,
        slug: &str,
    ) -> ServerResult<Option<Self>> {
        Ok(program_specs::table
            .filter(program_specs::dsl::hardware_spec_id.eq(hardware_spec_id))
            .filter(program_specs::dsl::slug.eq(slug))
            .get_result::<models::ProgramSpec>(conn)
            .optional()?
            .map(Self::from_model))
    }
}

impl NodeType for ProgramSpecNode {
    type Model = models::ProgramSpec;

    fn from_model(model: Self::Model) -> Self {
        Self {
            program_spec: model,
        }
    }

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
            .filter(
                program_specs::dsl::hardware_spec_id.eq(self.hardware_spec_id),
            )
            .select(dsl::count(program_specs::id))
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
            .filter(
                program_specs::dsl::hardware_spec_id.eq(self.hardware_spec_id),
            )
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
