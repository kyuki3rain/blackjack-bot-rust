-- Add up migration script here
CREATE TABLE blackjack_bot_rust_bonuses (
    id SERIAL PRIMARY KEY,
    amount INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE blackjack_bot_rust_user_bonuses (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    bonus_id INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (user_id) REFERENCES blackjack_bot_rust_users(id),
    FOREIGN KEY (bonus_id) REFERENCES blackjack_bot_rust_bonuses(id)
);