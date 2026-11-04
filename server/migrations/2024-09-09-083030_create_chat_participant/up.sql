CREATE TABLE chat_participant (
    chat_id INT NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
    member_id INT NOT NULL REFERENCES member(id) ON DELETE CASCADE,
    joined_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chat_participant_pkey PRIMARY KEY (chat_id, member_id),
    CONSTRAINT chat_participant_chat_id_positive CHECK (chat_id > 0),
    CONSTRAINT chat_participant_member_id_positive CHECK (member_id > 0)
);
CREATE INDEX chat_participant_member_id_idx ON chat_participant(member_id);
