CREATE OR REPLACE FUNCTION delete_dangling_program_records()
    RETURNS trigger AS
$$
BEGIN
    WITH pbs as (
        SELECT min(cpu_cycles) as cpu_cycles, min(instructions) as instructions, min(registers_used) as registers_used, min(stacks_used) as stacks_used
        FROM "user_program_records" as upr
        WHERE upr.user_id = NEW.user_id 
        AND upr.program_spec_id = NEW.program_spec_id
    )
    DELETE FROM "user_program_records" WHERE id in (
        SELECT id
        FROM "user_program_records" as upr, pbs
        WHERE upr.user_id = NEW.user_id AND upr.program_spec_id = NEW.program_spec_id
        AND upr.cpu_cycles > pbs.cpu_cycles
        AND upr.instructions > pbs.instructions
        AND upr.registers_used > pbs.registers_used
        AND upr.stacks_used > pbs.stacks_used
        AND upr.id NOT IN (
            -- get all referenced rows so they wont be deleted
            SELECT record_id FROM "user_programs" as up
            WHERE up.user_id = NEW.user_id AND up.program_spec_id = NEW.program_spec_id
        )
        AND upr.id <> NEW.id
    );
RETURN NULL;
END;
$$
LANGUAGE 'plpgsql';

CREATE TRIGGER delete_dangling_records
    AFTER INSERT
    ON "user_program_records"
    FOR EACH ROW
    EXECUTE PROCEDURE delete_dangling_program_records();
