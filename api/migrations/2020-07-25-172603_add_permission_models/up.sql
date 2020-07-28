CREATE TYPE role_type AS ENUM ('admin', 'spec_creator');
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name role_type NOT NULL UNIQUE,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE -- admins implicitly get all permissions
);

CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id),
    role_id UUID NOT NULL REFERENCES roles(id),
    UNIQUE(user_id, role_id)
);

CREATE TYPE permission_type AS ENUM (
    'create_specs',
    'modify_all_specs',
    'delete_all_specs',
    'view_all_user_programs'
);
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name permission_type NOT NULL UNIQUE
);

CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role_id UUID NOT NULL REFERENCES roles(id),
    permission_id UUID NOT NULL REFERENCES permissions(id),
    UNIQUE(role_id, permission_id)
);

INSERT INTO roles (name, is_admin) VALUES ('admin', TRUE);
INSERT INTO roles (name) VALUES ('spec_creator');
INSERT INTO permissions (name) VALUES
    ('create_specs'),
    ('modify_all_specs'),
    ('delete_all_specs'),
    ('view_all_user_programs');
INSERT INTO role_permissions (role_id, permission_id)
    SELECT roles.id, permissions.id FROM roles, permissions
    WHERE roles.name = 'spec_creator' and permissions.name = 'create_specs';
