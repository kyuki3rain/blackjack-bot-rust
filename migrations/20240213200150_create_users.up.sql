-- Add up migration script here
CREATE TABLE blackjack_bot_rust_users (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    balance INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
