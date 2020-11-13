ALTER TABLE user_program_records
    ADD COLUMN cost INTEGER GENERATED ALWAYS AS ((2 * cpu_cycles) + (3 * instructions) + (100 * registers_used) + (200 * stacks_used)) STORED NOT NULL;

CREATE INDEX user_program_records_program_spec_and_user_ids ON user_program_records (program_spec_id, user_id);

CREATE OR REPLACE FUNCTION delete_dangling_program_records ()
    RETURNS TRIGGER
    AS $$
BEGIN
    -- get the min for each stat so we know what this user's current personal bests are
    WITH pbs AS (
        SELECT
            min(cpu_cycles) AS cpu_cycles,
            min(instructions) AS instructions,
            min(registers_used) AS registers_used,
            min(stacks_used) AS stacks_used,
            min(cost) AS cost
        FROM
            "user_program_records"
        WHERE
            "user_program_records".user_id = NEW.user_id
            AND "user_program_records".program_spec_id = NEW.program_spec_id
    )
    DELETE FROM "user_program_records"
    WHERE id IN (
        SELECT
            id
        FROM
            "user_program_records",
            pbs
        WHERE
            -- Don't delete the record we just inserted since it won't be referenced yet
            "user_program_records".id <> NEW.id
            -- Only grab records by this user for this program spec
            AND "user_program_records".user_id = NEW.user_id
            AND "user_program_records".program_spec_id = NEW.program_spec_id
            -- only delete records that are not a pb for any of the stats we track
            AND "user_program_records".cpu_cycles > pbs.cpu_cycles
            AND "user_program_records".instructions > pbs.instructions
            AND "user_program_records".registers_used > pbs.registers_used
            AND "user_program_records".stacks_used > pbs.stacks_used
            AND "user_program_records".cost > pbs.cost
            -- get all referenced rows so they wont be deleted
            AND "user_program_records".id NOT IN (
                SELECT
                    record_id
                FROM
                    "user_programs"
                WHERE
                    "user_programs".user_id = NEW.user_id
                    AND "user_programs".program_spec_id = NEW.program_spec_id
            )
        );
    RETURN NULL;
END;
$$
LANGUAGE 'plpgsql';

CREATE TRIGGER delete_dangling_records
    AFTER INSERT ON "user_program_records"
    FOR EACH ROW
    EXECUTE PROCEDURE delete_dangling_program_records ();

