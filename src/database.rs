use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

pub enum UserId {
    Discord(u64),
    Name(String),
}

impl UserId {
    pub async fn get_from_db(&self, conn: &Pool<Postgres>) -> Result<i32, sqlx::Error> {
        let user_id = match self {
            UserId::Discord(id) => {
                let id: i64 = discord_id_to_i64(id.clone());
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
            UserId::Name(name) => {
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

pub fn discord_id_to_i64(id: u64) -> i64 {
    (id as i128 + i64::MIN as i128) as i64
}

pub async fn establish_connection() -> Result<Pool<Postgres>, sqlx::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn create_user(pool: &Pool<Postgres>, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO blackjack_bot_rust_users (name, balance)
        VALUES ($1, 0)
        "#,
        name
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_id_by_name(pool: &Pool<Postgres>, name: &str) -> Result<i32, sqlx::Error> {
    let user = sqlx::query!(
        r#"
        SELECT id
        FROM blackjack_bot_rust_users
        WHERE name = $1
        "#,
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(user.id)
}

pub async fn get_user_id_by_discord(
    pool: &Pool<Postgres>,
    discord_id: u64,
) -> Result<i32, sqlx::Error> {
    let discord_id = discord_id_to_i64(discord_id);

    let discord_user = sqlx::query!(
        r#"
        SELECT user_id
        FROM blackjack_bot_rust_discord_users
        WHERE discord_id = $1
        "#,
        discord_id
    )
    .fetch_one(pool)
    .await?;

    Ok(discord_user.user_id)
}

pub async fn get_username_by_discord(
    pool: &Pool<Postgres>,
    discord_id: u64,
) -> Result<String, sqlx::Error> {
    let discord_id = discord_id_to_i64(discord_id);

    let user = sqlx::query!(
        r#"
        SELECT name
        FROM blackjack_bot_rust_users
        WHERE id = (
            SELECT user_id
            FROM blackjack_bot_rust_discord_users
            WHERE discord_id = $1
        )
        "#,
        discord_id
    )
    .fetch_one(pool)
    .await?;

    Ok(user.name)
}

pub async fn create_discord_user(
    pool: &Pool<Postgres>,
    discord_user_id: u64,
    name: &str,
) -> Result<(), sqlx::Error> {
    let discord_user_id = discord_id_to_i64(discord_user_id);

    let user_id = match get_user_id_by_name(pool, name).await {
        Ok(id) => id,
        Err(_) => {
            create_user(pool, name).await?;
            get_user_id_by_name(pool, name).await?
        }
    };

    sqlx::query!(
        r#"
        INSERT INTO blackjack_bot_rust_discord_users (discord_id, user_id)
        VALUES ($1, $2)
        "#,
        discord_user_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}