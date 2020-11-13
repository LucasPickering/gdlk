DROP TRIGGER delete_dangling_records;
DROP FUNCTION delete_dangling_program_records;
DROP INDEX user_program_records_program_spec_and_user_ids;
ALTER TABLE user_program_records DROP COLUMN cost;

