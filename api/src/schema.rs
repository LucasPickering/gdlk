#![allow(unused_imports)]

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    hardware_specs (id) {
        id -> Uuid,
        slug -> Varchar,
        name -> Varchar,
        num_registers -> Int4,
        num_stacks -> Int4,
        max_stack_length -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    permissions (id) {
        id -> Uuid,
        name -> Permission_type,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    program_specs (id) {
        id -> Uuid,
        slug -> Varchar,
        name -> Varchar,
        description -> Text,
        hardware_spec_id -> Uuid,
        input -> Array<Int4>,
        expected_output -> Array<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    role_permissions (id) {
        id -> Uuid,
        role_id -> Uuid,
        permission_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    roles (id) {
        id -> Uuid,
        name -> Role_type,
        is_admin -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    user_programs (id) {
        id -> Uuid,
        user_id -> Uuid,
        program_spec_id -> Uuid,
        file_name -> Text,
        source_code -> Text,
        created -> Timestamptz,
        last_modified -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    user_providers (id) {
        id -> Uuid,
        sub -> Varchar,
        provider_name -> Text,
        user_id -> Nullable<Uuid>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    user_roles (id) {
        id -> Uuid,
        user_id -> Uuid,
        role_id -> Uuid,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::models::sql_types::*;

    users (id) {
        id -> Uuid,
        username -> Varchar,
    }
}

joinable!(program_specs -> hardware_specs (hardware_spec_id));
joinable!(role_permissions -> permissions (permission_id));
joinable!(role_permissions -> roles (role_id));
joinable!(user_programs -> program_specs (program_spec_id));
joinable!(user_programs -> users (user_id));
joinable!(user_providers -> users (user_id));
joinable!(user_roles -> roles (role_id));
joinable!(user_roles -> users (user_id));

allow_tables_to_appear_in_same_query!(
    hardware_specs,
    permissions,
    program_specs,
    role_permissions,
    roles,
    user_programs,
    user_providers,
    user_roles,
    users,
);
