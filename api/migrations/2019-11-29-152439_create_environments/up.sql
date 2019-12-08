CREATE TABLE environments (
  id SERIAL PRIMARY KEY,
  num_registers INTEGER NOT NULL CHECK(num_registers >= 1),
  num_stacks INTEGER NOT NULL CHECK(num_stacks >= 0),
  max_stack_length INTEGER NOT NULL CHECK(num_stacks >= 0),
  input INTEGER[] NOT NULL,
  expected_output INTEGER[] NOT NULL
);
