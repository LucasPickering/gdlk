table! {
    hardware_specs (id) {
        id -> Uuid,
        name -> Varchar,
        num_registers -> Int4,
        num_stacks -> Int4,
        max_stack_length -> Int4,
        slug -> Varchar,
    }
}

table! {
    permissions (id) {
        id -> Uuid,
        name -> Varchar,
        slug -> Varchar,
    }
}

table! {
    program_specs (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Text,
        hardware_spec_id -> Uuid,
        input -> Array<Int4>,
        expected_output -> Array<Int4>,
        slug -> Varchar,
    }
}

table! {
    role_permissions (id) {
        id -> Uuid,
        role_id -> Uuid,
        permission_id -> Uuid,
    }
}

table! {
    roles (id) {
        id -> Uuid,
        name -> Varchar,
        is_admin -> Bool,
        slug -> Varchar,
    }
}

table! {
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
    user_providers (id) {
        id -> Uuid,
        sub -> Varchar,
        provider_name -> Text,
        user_id -> Nullable<Uuid>,
    }
}

table! {
    user_roles (id) {
        id -> Uuid,
        user_id -> Uuid,
        role_id -> Uuid,
    }
}

table! {
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
