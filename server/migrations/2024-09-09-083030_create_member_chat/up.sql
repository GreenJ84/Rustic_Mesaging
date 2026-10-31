CREATE TABLE member_chat (
    chat_id INT NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT member_chat_pkey PRIMARY KEY (chat_id, member_id),
    CONSTRAINT member_chat_chat_id_positive CHECK (chat_id > 0),
    CONSTRAINT member_chat_member_id_positive CHECK (member_id > 0)
);
CREATE INDEX member_chat_member_id_idx ON member_chat(member_id);
