CREATE TABLE server_membership (
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    server_id INT NOT NULL REFERENCES server(id) ON DELETE CASCADE,
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY(member_id, server_id)
);

CREATE INDEX member_servers ON server_membership(member_id);
CREATE INDEX server_members ON server_membership(server_id);
