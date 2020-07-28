use diesel_derive_enum::DbEnum;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, DbEnum)]
#[DieselType = "Permission_type"]
pub enum PermissionType {
    /// User can create new hardware and program specs
    CreateSpecs,

    /// User can modify all existing hardware and program specs, regardless of
    /// whether or not they are the owner.
    ModifyAllSpecs,

    /// User can delete all existing hardware and program specs, regardless of
    /// whether or not they are the owner.
    DeleteAllSpecs,

    /// User can view all existing user programs, not just their own.
    ViewAllUserPrograms,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, DbEnum)]
#[DieselType = "Role_type"]
pub enum RoleType {
    /// These users can perform any action.
    Admin,

    /// These users can create specs. Once we start attaching the creator to
    /// each spec then we can allow these users to modify their own specs, but
    /// for now they can only create them.
    SpecCreator,
}
