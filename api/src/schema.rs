table! {
    environments (id) {
        id -> Int4,
        num_registers -> Int4,
        num_stacks -> Int4,
        max_stack_length -> Int4,
        input -> Array<Int4>,
        expected_output -> Array<Int4>,
    }
}
