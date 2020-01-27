table! {
    hardware_specs (id) {
        id -> Int4,
        slug -> Varchar,
        num_registers -> Int4,
        num_stacks -> Int4,
        max_stack_length -> Int4,
    }
}

table! {
    program_specs (id) {
        id -> Int4,
        slug -> Varchar,
        hardware_spec_id -> Int4,
        input -> Array<Int4>,
        expected_output -> Array<Int4>,
    }
}

table! {
    user_programs (id) {
        id -> Int4,
        user_id -> Int4,
        program_spec_id -> Int4,
        file_name -> Text,
        source_code -> Text,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
    }
}

joinable!(program_specs -> hardware_specs (hardware_spec_id));
joinable!(user_programs -> program_specs (program_spec_id));
joinable!(user_programs -> users (user_id));

allow_tables_to_appear_in_same_query!(
    hardware_specs,
    program_specs,
    user_programs,
    users,
);
