CREATE TABLE friend (
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    friend_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT friend_members_must_differ
        CHECK (member_id <> friend_id),

    CONSTRAINT friend_ids_are_canonical
        CHECK (member_id < friend_id),

    CONSTRAINT friend_pkey
        PRIMARY KEY (member_id, friend_id)
);

CREATE INDEX idx_friend ON friend(friend_id);
