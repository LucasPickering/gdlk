CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(slug) > 0),
    name VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(name) > 0),
    is_admin BOOLEAN NOT NULL DEFAULT FALSE -- admins implicitly get all permissions
);
-- autogenerate slug from name
CREATE TRIGGER "t_roles_insert" BEFORE INSERT ON "roles"
FOR EACH ROW EXECUTE PROCEDURE set_slug_from_name();

CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    role_id UUID NOT NULL REFERENCES roles(id),
    UNIQUE(user_id, role_id)
);

CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(slug) > 0),
    name VARCHAR(50) NOT NULL UNIQUE CHECK (char_length(name) > 0)
);
-- autogenerate slug from name
CREATE TRIGGER "t_permissions_insert" BEFORE INSERT ON "permissions"
FOR EACH ROW EXECUTE PROCEDURE set_slug_from_name();

CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role_id UUID NOT NULL REFERENCES roles(id),
    permission_id UUID NOT NULL REFERENCES permissions(id),
    UNIQUE(role_id, permission_id)
);

INSERT INTO roles (name, is_admin) VALUES
    ('Admin', TRUE),
    ('Spec Creator', FALSE);
INSERT INTO permissions (name) VALUES
    ('Create Specs'),
    ('Modify All Specs'),
    ('Delete All Specs'),
    ('View All User Programs');
INSERT INTO role_permissions (role_id, permission_id)
    SELECT roles.id, permissions.id FROM roles, permissions
    WHERE roles.name = 'Spec Creator' and permissions.name = 'Create Specs';
