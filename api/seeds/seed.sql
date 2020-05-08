WITH user_row AS (
    INSERT INTO users (username) VALUES ('user1') RETURNING id
),
hw_spec_rows AS (
    INSERT INTO hardware_specs (slug, num_registers, num_stacks, max_stack_length)
        VALUES ('k100', 1, 0, 0), ('k210', 2, 1, 10), ('k320', 3, 2, 32)
        RETURNING id, slug
),

-- Program specs for k100
prog_spec1_row AS (
    INSERT INTO program_specs (slug, hardware_spec_id, input, expected_output)
        SELECT 'prog1', hw_spec_rows.id, '{1,2,3}', '{1,2,3}'
        FROM hw_spec_rows WHERE hw_spec_rows.slug = 'k100'
        RETURNING id
),
prog_spec2_row AS (
    INSERT INTO program_specs (slug, hardware_spec_id, input, expected_output)
        SELECT 'prog2', hw_spec_rows.id, '{1,2,3}', '{2,4,6}'
        FROM hw_spec_rows WHERE hw_spec_rows.slug = 'k100'
        RETURNING id
),

-- Program specs for k320
prog_spec3_row AS (
    INSERT INTO program_specs (slug, hardware_spec_id, input, expected_output)
        SELECT 'prog1', hw_spec_rows.id, '{-32768,32767}', '{0,0}'
        FROM hw_spec_rows WHERE hw_spec_rows.slug = 'k320'
        RETURNING id
),
prog_spec4_row AS (
    INSERT INTO program_specs (slug, hardware_spec_id, input, expected_output)
        SELECT 'sort', hw_spec_rows.id, '{9,3,8,4,5,1,3,8,9,5,2,10,4,1,8}', '{1,1,2,3,3,4,4,5,5,8,8,8,9,9,10}'
        FROM hw_spec_rows WHERE hw_spec_rows.slug = 'k320'
        RETURNING id
)

--- Program specs

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
