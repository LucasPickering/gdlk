use std::str::FromStr;

use crate::{
    error::{ResponseError, ResponseResult, ServerError},
    schema::user_program_pbs,
};
use diesel::{
    prelude::*, query_builder::InsertStatement, Identifiable, Queryable,
};
use uuid::Uuid;

/// A mapping of all stat variants to the corresponding name used in the DB
const STAT_NAME_MAPPING: &[(StatType, &str)] = &[
    (StatType::CpuCycles, "cpu_cycles"),
    (StatType::Instructions, "instructions"),
    (StatType::RegistersUsed, "registers_used"),
    (StatType::StacksUsed, "stacks_used"),
];

/// The different program statistics that we track.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum StatType {
    CpuCycles,
    Instructions,
    RegistersUsed,
    StacksUsed,
}

impl StatType {
    pub fn to_str(self) -> &'static str {
        for (role_type, name) in STAT_NAME_MAPPING {
            if self == *role_type {
                return name;
            }
        }
        panic!("Missing name for stat type: {:?}", self);
    }

    pub fn all_types() -> impl Iterator<Item = Self> {
        STAT_NAME_MAPPING.iter().map(|(stat_type, _)| *stat_type)
    }
}

impl FromStr for StatType {
    type Err = ResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (role_type, name) in STAT_NAME_MAPPING {
            if s == *name {
                return Ok(*role_type);
            }
        }

        // Unknown value
        Err(ServerError::InvalidDbValue {
            column: Box::new(user_program_pbs::columns::stat),
            value: s.into(),
        }
        .into())
    }
}

/// A user's Personal Best record for a particular program spec.
#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
#[table_name = "user_program_pbs"]
pub struct UserProgramPb {
    /// DB primary key
    pub id: Uuid,
    /// Foreign key to the user
    pub user_id: Uuid,
    /// The program spec for which this solution is written
    pub program_spec_id: Uuid,
    /// Statistical record for the user's solution that used the fewest number
    /// of CPU cycles.
    pub record_id: Uuid,
    /// The name of the statistic that this row is a PB for. Maps to one of
    /// [StatType]. Use [Self::stat_type] to parse this.
    pub stat: String,
}

impl UserProgramPb {
    /// Parse the `stat` value of this model into a [StatType].
    pub fn stat_type(&self) -> ResponseResult<StatType> {
        self.stat.parse()
    }
}

#[derive(Clone, Debug, Insertable)]
#[table_name = "user_program_pbs"]
pub struct NewUserProgramPb<'a> {
    pub user_id: Uuid,
    pub program_spec_id: Uuid,
    pub record_id: Uuid,
    pub stat: &'a str,
}

impl NewUserProgramPb<'_> {
    /// Insert this object into the `user_program_pbs` DB table.
    pub fn insert(
        self,
    ) -> InsertStatement<
        user_program_pbs::table,
        <Self as Insertable<user_program_pbs::table>>::Values,
    > {
        self.insert_into(user_program_pbs::table)
    }
}
