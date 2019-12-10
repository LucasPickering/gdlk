CREATE TABLE hardware_specs (
    id SERIAL PRIMARY KEY,
    num_registers INTEGER NOT NULL CHECK(num_registers >= 1),
    num_stacks INTEGER NOT NULL CHECK(num_stacks >= 0),
    max_stack_length INTEGER NOT NULL CHECK(num_stacks >= 0)
);

CREATE TABLE program_specs (
    id SERIAL PRIMARY KEY,
    hardware_spec_id INTEGER NOT NULL REFERENCES hardware_specs(id),
    input INTEGER[] NOT NULL,
    expected_output INTEGER[] NOT NULL
);
