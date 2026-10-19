CREATE TABLE system_role (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    authority_level INT NOT NULL UNIQUE,
    system_managed BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE TABLE member_system_role (
    member_id INT NOT NULL UNIQUE REFERENCES member(id) ON DELETE CASCADE,
    role_id INT NOT NULL REFERENCES system_role(id) ON DELETE RESTRICT,
    assigned_by INT REFERENCES member(id) ON DELETE SET NULL,
    assigned_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (member_id, role_id)
);
