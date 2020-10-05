-- Metrics to measure the performance of a user_program
CREATE TABLE user_program_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_code TEXT NOT NULL,
    cpu_cycles INTEGER NOT NULL CHECK(cpu_cycles >= 0),
    instructions INTEGER NOT NULL CHECK(instructions >= 0),
    registers_used INTEGER NOT NULL CHECK(registers_used >= 0),
    stacks_used INTEGER NOT NULL CHECK(stacks_used >= 0),
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- A user's best solution for a program in a particular stat
-- pb for "personal best"
CREATE TABLE user_program_pbs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    program_spec_id UUID NOT NULL REFERENCES program_specs(id),
    record_id UUID NOT NULL REFERENCES user_program_records(id),
    stat TEXT NOT NULL CHECK(stat IN (
        'cpu_cycles', 'instructions', 'registers_used', 'stacks_used'
    )),
    UNIQUE(user_id, program_spec_id, stat)
);

-- Each solution tracks stats for its last run. This column should be populated
-- after every successful run, and cleared any time the source code changes.
ALTER TABLE user_programs
    ADD COLUMN record_id UUID REFERENCES user_program_records(id);
