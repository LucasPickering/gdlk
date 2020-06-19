table! {
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
    user_programs (id) {
        id -> Uuid,
        user_id -> Uuid,
        program_spec_id -> Uuid,
        file_name -> Text,
        source_code -> Text,
        last_modified -> Nullable<Timestamptz>,
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
    users (id) {
        id -> Uuid,
        username -> Varchar,
    }
}

joinable!(program_specs -> hardware_specs (hardware_spec_id));
joinable!(user_programs -> program_specs (program_spec_id));
joinable!(user_programs -> users (user_id));
joinable!(user_providers -> users (user_id));

allow_tables_to_appear_in_same_query!(
    hardware_specs,
    program_specs,
    user_programs,
    user_providers,
    users,
);
