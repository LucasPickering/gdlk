CREATE TABLE hardware_specs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(20) NOT NULL UNIQUE,
    num_registers INTEGER NOT NULL CHECK(num_registers >= 1),
    num_stacks INTEGER NOT NULL CHECK(num_stacks >= 0),
    max_stack_length INTEGER NOT NULL CHECK(num_stacks >= 0)
);

CREATE TABLE program_specs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(20) NOT NULL,
    hardware_spec_id UUID NOT NULL REFERENCES hardware_specs(id),
    input SMALLINT[] NOT NULL,
    expected_output SMALLINT[] NOT NULL,
    UNIQUE(slug, hardware_spec_id)
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(20) NOT NULL UNIQUE
);

CREATE TABLE user_programs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    program_spec_id UUID NOT NULL REFERENCES program_specs(id),
    file_name TEXT NOT NULL,
    source_code TEXT NOT NULL,
    UNIQUE(user_id, program_spec_id, file_name)
);
