CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(20) NOT NULL UNIQUE
);

CREATE TABLE user_programs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    program_spec_id INTEGER NOT NULL REFERENCES program_specs(id),
    file_name TEXT NOT NULL,
    source_code TEXT NOT NULL,
    UNIQUE(user_id, program_spec_id, file_name)
);
