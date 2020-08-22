CREATE TABLE hardware_specs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(slug) > 0),
    name VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(name) > 0),
    num_registers INTEGER NOT NULL CHECK(num_registers >= 1),
    num_stacks INTEGER NOT NULL CHECK(num_stacks >= 0),
    max_stack_length INTEGER NOT NULL CHECK(num_stacks >= 0)
);
-- autogenerate slug from name
CREATE TRIGGER "t_hardware_specs_insert" BEFORE INSERT ON "hardware_specs"
FOR EACH ROW EXECUTE PROCEDURE set_slug_from_name();

CREATE TABLE program_specs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(50) NOT NULL CHECK (char_length(slug) > 0),
    name VARCHAR(50) NOT NULL CHECK (char_length(name) > 0),
    description TEXT NOT NULL,
    hardware_spec_id UUID NOT NULL REFERENCES hardware_specs(id),
    input INT[] NOT NULL CHECK (array_length(input, 1) <= 256),
    expected_output INT[] NOT NULL CHECK (array_length(input, 1) <= 256),
    UNIQUE(slug, hardware_spec_id),
    UNIQUE(name, hardware_spec_id)
);
-- autogenerate slug from name
CREATE TRIGGER "t_program_specs_insert" BEFORE INSERT ON "program_specs"
FOR EACH ROW EXECUTE PROCEDURE set_slug_from_name();

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(20) NOT NULL UNIQUE CHECK (char_length(username) > 0)
);

CREATE TABLE user_programs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    program_spec_id UUID NOT NULL REFERENCES program_specs(id),
    file_name TEXT NOT NULL CHECK (char_length(file_name) > 0),
    source_code TEXT NOT NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_modified TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, program_spec_id, file_name)
);
CREATE TRIGGER
  update_last_modified
BEFORE UPDATE ON
  user_programs
FOR EACH ROW EXECUTE PROCEDURE
  update_last_modified();

CREATE TABLE user_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sub VARCHAR(255) NOT NULL,
    provider_name Text NOT NULL,
    user_id UUID references users(id), -- can be null if the user has not set their username yet
    UNIQUE(sub, provider_name)
)
