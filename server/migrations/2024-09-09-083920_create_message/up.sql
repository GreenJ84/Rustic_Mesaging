CREATE TABLE message (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    sender_id INT NOT NULL DEFAULT 1
        REFERENCES member(id) ON DELETE SET DEFAULT,
    chat_id INT NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT message_content_not_blank CHECK (length(btrim(content)) > 0),
    CONSTRAINT message_content_length CHECK (length(content) <= 2000)
);

CREATE INDEX message_chat_id_created_at_idx ON message(chat_id, created_at DESC, id DESC);
