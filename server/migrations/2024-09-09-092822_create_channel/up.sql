CREATE TABLE channel (
    id SERIAL PRIMARY KEY,
    server_id INT NOT NULL REFERENCES server(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX server_channel ON channel(server_id);