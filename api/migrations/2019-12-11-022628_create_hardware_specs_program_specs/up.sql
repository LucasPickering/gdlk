CREATE TABLE hardware_specs (
    id SERIAL PRIMARY KEY,
    slug VARCHAR(20) NOT NULL UNIQUE,
    num_registers INTEGER NOT NULL CHECK(num_registers >= 1),
    num_stacks INTEGER NOT NULL CHECK(num_stacks >= 0),
    max_stack_length INTEGER NOT NULL CHECK(num_stacks >= 0)
);

CREATE TABLE program_specs (
    id SERIAL PRIMARY KEY,
    slug VARCHAR(20) NOT NULL,
    hardware_spec_id INTEGER NOT NULL REFERENCES hardware_specs(id),
    input INTEGER[] NOT NULL,
    expected_output INTEGER[] NOT NULL,
    UNIQUE(slug, hardware_spec_id)
);
