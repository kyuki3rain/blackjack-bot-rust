use sqlx::{Pool, Postgres};

#[derive(Debug, Clone)]
pub enum UserId {
    Discord(i32),
    Cli(String),
    Name(String),
}

impl UserId {
    pub async fn get_from_db(&self, conn: &Pool<Postgres>) -> Result<i32, sqlx::Error> {
        let user_id = match self {
            UserId::Discord(id) => {
                sqlx::query!(
                    r#"
                    SELECT user_id
                    FROM blackjack_bot_rust_discord_users
                    WHERE discord_id = $1
                    "#,
                    id
                )
                .fetch_one(conn)
                .await?
                .user_id
            }
            UserId::Cli(name) | UserId::Name(name) => {
                sqlx::query!(
                    r#"
                    SELECT id
                    FROM blackjack_bot_rust_users
                    WHERE name = $1
                    "#,
                    name
                )
                .fetch_one(conn)
                .await?
                .id
            }
        };

        Ok(user_id)
    }
}
