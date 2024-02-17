-- Add up migration script here

CREATE TABLE blackjack_bot_rust_discord_users (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    discord_id INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES blackjack_bot_rust_users(id)
);