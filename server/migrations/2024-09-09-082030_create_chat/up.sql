CREATE TABLE chat (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT chat_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT chat_name_length CHECK (length(btrim(name)) BETWEEN 3 AND 80)
);
