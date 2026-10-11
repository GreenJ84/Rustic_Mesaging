CREATE TABLE role (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
INSERT INTO role (name) VALUES
    ('admin'),
    ('moderator'),
    ('user');

CREATE TABLE member_role (
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    role_id INT NOT NULL REFERENCES role(id) ON DELETE CASCADE,
    PRIMARY KEY (member_id, role_id)
);