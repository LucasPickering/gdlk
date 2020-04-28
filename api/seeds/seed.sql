WITH user_row AS (
    INSERT INTO users (username) VALUES ('user1') RETURNING id
),
hw_spec_rows AS (
    INSERT INTO hardware_specs (slug, num_registers, num_stacks, max_stack_length)
        VALUES ('hw1', 1, 0, 0), ('hw2', 2, 1, 10), ('hw3', 5, 2, 50)
        RETURNING id, slug
),

-- Program specs for hw1
prog_spec1_row AS (
    INSERT INTO program_specs (slug, hardware_spec_id, input, expected_output)
        SELECT 'prog1', hw_spec_rows.id, '{1,2,3}', '{1,2,3}'
        FROM hw_spec_rows WHERE hw_spec_rows.slug = 'hw1'
        RETURNING id
),
prog_spec2_row AS (
    INSERT INTO program_specs (slug, hardware_spec_id, input, expected_output)
        SELECT 'prog2', hw_spec_rows.id, '{1,2,3}', '{2,4,6}'
        FROM hw_spec_rows WHERE hw_spec_rows.slug = 'hw1'
        RETURNING id
),

-- Program specs for hw3
prog_spec3_row AS (
    INSERT INTO program_specs (slug, hardware_spec_id, input, expected_output)
        SELECT 'prog1', hw_spec_rows.id, '{-32768,32767}', '{0,0}'
        FROM hw_spec_rows WHERE hw_spec_rows.slug = 'hw3'
        RETURNING id
)

INSERT INTO user_programs (user_id, program_spec_id, file_name, source_code)
    SELECT
        user_row.id, prog_spec1_row.id, 'program.gdlk',
        'READ RX0
WRITE RX0
READ RX0
WRITE RX0
READ RX0
WRITE RX0
'
    FROM user_row, prog_spec1_row;
