DROP TRIGGER "t_hardware_specs_insert" ON hardware_specs;
ALTER TABLE hardware_specs DROP COLUMN slug;
ALTER TABLE hardware_specs ADD COLUMN slug VARCHAR(50) GENERATED ALWAYS AS (slugify("name")) STORED NOT NULL UNIQUE CHECK (char_length(slug) > 0);



DROP TRIGGER "t_program_specs_insert" ON program_specs;
ALTER TABLE program_specs DROP COLUMN slug;
ALTER TABLE program_specs ADD COLUMN slug VARCHAR(50) GENERATED ALWAYS AS (slugify("name")) STORED NOT NULL UNIQUE CHECK (char_length(slug) > 0);



DROP TRIGGER "t_roles_insert" ON roles;
ALTER TABLE roles DROP COLUMN slug;
ALTER TABLE roles ADD COLUMN slug VARCHAR(50) GENERATED ALWAYS AS (slugify("name")) STORED NOT NULL UNIQUE CHECK (char_length(slug) > 0);


DROP TRIGGER "t_permissions_insert" ON permissions;
ALTER TABLE permissions DROP COLUMN slug;
ALTER TABLE permissions ADD COLUMN slug VARCHAR(50) GENERATED ALWAYS AS (slugify("name")) STORED NOT NULL UNIQUE CHECK (char_length(slug) > 0);
