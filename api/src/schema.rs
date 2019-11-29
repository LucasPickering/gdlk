table! {
    environments (id) {
        id -> Int4,
        num_stacks -> Int4,
        max_stack_size -> Nullable<Int4>,
        input -> Array<Int4>,
        expected_output -> Array<Int4>,
    }
}
