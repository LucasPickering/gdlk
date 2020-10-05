ALTER TABLE hardware_specs DROP COLUMN slug;
ALTER TABLE hardware_specs ADD COLUMN slug VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(slug) > 0);
CREATE TRIGGER "t_hardware_specs_insert" BEFORE INSERT ON "hardware_specs"
FOR EACH ROW EXECUTE PROCEDURE set_slug_from_name();

ALTER TABLE program_specs DROP COLUMN slug;
ALTER TABLE program_specs ADD COLUMN slug VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(slug) > 0);
CREATE TRIGGER "t_program_specs_insert" BEFORE INSERT ON "program_specs"
FOR EACH ROW EXECUTE PROCEDURE set_slug_from_name();

ALTER TABLE roles DROP COLUMN slug;
ALTER TABLE roles ADD COLUMN slug VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(slug) > 0);
CREATE TRIGGER "t_roles_insert" BEFORE INSERT ON "roles"
FOR EACH ROW EXECUTE PROCEDURE set_slug_from_name();

ALTER TABLE permissions DROP COLUMN slug;
ALTER TABLE permissions ADD COLUMN slug VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(slug) > 0);
CREATE TRIGGER "t_permissions_insert" BEFORE INSERT ON "permissions"
FOR EACH ROW EXECUTE PROCEDURE set_slug_from_name();
