use crate::{
    error::ResponseResult,
    models,
    schema::{hardware_specs, program_specs},
    server::gql::{
        internal::{GenericEdge, NodeType},
        program_spec::{ProgramSpecConnection, ProgramSpecNode},
        ConnectionPageParams, Context, CreateHardwareSpecPayloadFields, Cursor,
        HardwareSpecConnectionFields, HardwareSpecEdgeFields,
        HardwareSpecNodeFields, PageInfo, UpdateHardwareSpecPayloadFields,
    },
    util,
};
use diesel::{
    dsl, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    QueryResult, RunQueryDsl,
};
use gdlk::Valid;
use juniper::ID;
use juniper_from_schema::{QueryTrail, Walked};
use std::convert::TryInto;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct HardwareSpecNode {
    pub hardware_spec: models::HardwareSpec,
}

impl From<models::HardwareSpec> for HardwareSpecNode {
    fn from(model: models::HardwareSpec) -> Self {
        Self {
            hardware_spec: model,
        }
    }
}

impl NodeType for HardwareSpecNode {
    type Model = models::HardwareSpec;

    fn find(conn: &PgConnection, id: Uuid) -> QueryResult<Self::Model> {
        hardware_specs::table.find(id).get_result(conn)
    }
}

impl HardwareSpecNodeFields for HardwareSpecNode {
    fn field_id(&self, _executor: &juniper::Executor<'_, Context>) -> ID {
        util::uuid_to_gql_id(self.hardware_spec.id)
    }

    fn field_slug(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &String {
        &self.hardware_spec.slug
    }

    fn field_name(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> &String {
        &self.hardware_spec.name
    }

    fn field_num_registers(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> i32 {
        self.hardware_spec.num_registers
    }

    fn field_num_stacks(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> i32 {
        self.hardware_spec.num_stacks
    }

    fn field_max_stack_length(
        &self,
        _executor: &juniper::Executor<'_, Context>,
    ) -> i32 {
        self.hardware_spec.max_stack_length
    }

    fn field_program_spec(
        &self,
        executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, ProgramSpecNode, Walked>,
        slug: String,
    ) -> ResponseResult<Option<ProgramSpecNode>> {
        // Get program spec by hardware spec + slug
        Ok(
            models::ProgramSpec::filter_by_hardware_spec(self.hardware_spec.id)
                .filter(program_specs::dsl::slug.eq(&slug))
                .get_result::<models::ProgramSpec>(
                    &executor.context().get_db_conn()?,
                )
                .optional()?
                .map(ProgramSpecNode::from),
        )
    }

    fn field_program_specs(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, ProgramSpecConnection, Walked>,
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> ResponseResult<ProgramSpecConnection> {
        ProgramSpecConnection::new(self.hardware_spec.id, first, after)
    }
}

pub type HardwareSpecEdge = GenericEdge<HardwareSpecNode>;

impl HardwareSpecEdgeFields for HardwareSpecEdge {
    fn field_node(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, HardwareSpecNode, Walked>,
    ) -> &HardwareSpecNode {
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
pub struct HardwareSpecConnection {
    page_params: Valid<ConnectionPageParams>,
}

impl HardwareSpecConnection {
    pub fn new(
        first: Option<i32>,
        after: Option<Cursor>,
    ) -> ResponseResult<Self> {
        Ok(Self {
            page_params: ConnectionPageParams::new(first, after)?,
        })
    }

    fn get_total_count(&self, context: &Context) -> ResponseResult<i32> {
        match hardware_specs::table
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
    ) -> ResponseResult<Vec<HardwareSpecEdge>> {
        let offset = self.page_params.offset();

        // Load data from the query
        let mut query =
            hardware_specs::table.offset(offset.into()).into_boxed();
        // Conditionally include limit param
        if let Some(limit) = self.page_params.limit() {
            query = query.limit(limit.into());
        }

        let rows: Vec<models::HardwareSpec> =
            query.get_results(&context.get_db_conn()?)?;

        Ok(HardwareSpecEdge::from_db_rows(rows, offset))
    }
}

impl HardwareSpecConnectionFields for HardwareSpecConnection {
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
        _trail: &QueryTrail<'_, HardwareSpecEdge, Walked>,
    ) -> ResponseResult<Vec<HardwareSpecEdge>> {
        self.get_edges(executor.context())
    }
}

pub struct CreateHardwareSpecPayload {
    pub hardware_spec: models::HardwareSpec,
}

impl CreateHardwareSpecPayloadFields for CreateHardwareSpecPayload {
    fn field_hardware_spec_edge(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, HardwareSpecEdge, Walked>,
    ) -> HardwareSpecEdge {
        GenericEdge::from_db_row(self.hardware_spec.clone(), 0)
    }
}

pub struct UpdateHardwareSpecPayload {
    pub hardware_spec: Option<models::HardwareSpec>,
}

impl UpdateHardwareSpecPayloadFields for UpdateHardwareSpecPayload {
    fn field_hardware_spec_edge(
        &self,
        _executor: &juniper::Executor<'_, Context>,
        _trail: &QueryTrail<'_, HardwareSpecEdge, Walked>,
    ) -> Option<HardwareSpecEdge> {
        self.hardware_spec
            .as_ref()
            .map(|row| GenericEdge::from_db_row(row.clone(), 0))
    }
}
