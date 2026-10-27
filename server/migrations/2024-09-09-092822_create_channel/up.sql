CREATE TABLE channel (
    id SERIAL PRIMARY KEY,
    server_id INT NOT NULL REFERENCES server(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX channel_server_name_unique
ON channel (server_id, lower(name));