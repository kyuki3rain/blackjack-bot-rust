-- Add up migration script here

CREATE TABLE blackjack_bot_rust_tables (
    id SERIAL PRIMARY KEY,
    discord_channel_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
