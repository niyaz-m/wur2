-- Add migration script here
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE channels (
    id BIGSERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    channel_id BIGINT REFERENCES channels(id),
    sender_id BIGINT REFERENCES users(id) NOT NULL,
    text TEXT,
    file_url TEXT,
    file_type TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_messages_channel_time
ON messages(channel_id, created_at);
