CREATE TABLE message (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL,
    sender_id INT DEFAULT 1 NOT NULL
        REFERENCES member(id) ON DELETE SET DEFAULT,
    chat_id INT NOT NULL REFERENCES chat(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX chat_messages ON message(chat_id);