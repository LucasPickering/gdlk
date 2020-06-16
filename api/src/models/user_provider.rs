use crate::schema::user_providers;
use diesel::{
    dsl,
    expression::{bound::Bound, operators},
    prelude::*,
    query_builder::InsertStatement,
    sql_types::Text,
    Queryable,
};
use uuid::Uuid;

pub type WithSub<'a> =
    dsl::Eq<user_providers::columns::sub, Bound<Text, &'a str>>;

pub type WithProvider<'a> =
    dsl::Eq<user_providers::columns::provider_name, Bound<Text, &'a str>>;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "user_providers"]
pub struct UserProvider {
    pub id: Uuid,
    pub sub: String,
    pub provider_name: String,
    pub user_id: Option<Uuid>,
}

impl UserProvider {
    pub fn filter_by_sub_and_provider<'a>(
        sub: &'a str,
        provider: &'a str,
    ) -> dsl::Filter<
        user_providers::table,
        operators::And<WithSub<'a>, WithProvider<'a>>,
    > {
        user_providers::table
            .filter(user_providers::dsl::sub.eq(sub))
            .filter(user_providers::dsl::provider_name.eq(provider))
    }
}

#[derive(Debug, Default, PartialEq, Insertable)]
#[table_name = "user_providers"]
pub struct NewUserProvider<'a> {
    pub sub: &'a str,
    pub provider_name: &'a str,
}

impl NewUserProvider<'_> {
    /// Insert this object into the `user_providers` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<
        user_providers::table,
        <Self as Insertable<user_providers::table>>::Values,
    > {
        self.insert_into(user_providers::table)
    }
}
