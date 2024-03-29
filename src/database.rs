use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

pub enum UserId {
    Discord(u64),
    Name(String),
}

impl UserId {
    pub async fn get_user_id(&self, conn: &Pool<Postgres>) -> Result<i32, sqlx::Error> {
        let user_id = match self {
            UserId::Discord(id) => {
                let id: i64 = discord_id_to_i64(*id);
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

pub async fn create_user(pool: &Pool<Postgres>, name: String) -> Result<(), sqlx::Error> {
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
    name: String,
) -> Result<(), sqlx::Error> {
    let discord_user_id = discord_id_to_i64(discord_user_id);

    let user_id = match UserId::Name(name.clone()).get_user_id(pool).await {
        Ok(id) => id,
        Err(_) => {
            create_user(pool, name.clone()).await?;
            UserId::Name(name).get_user_id(pool).await?
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

pub async fn get_balance(pool: &Pool<Postgres>, user_id: UserId) -> Result<i32, sqlx::Error> {
    let user_id = user_id.get_user_id(pool).await?;

    let balance = sqlx::query!(
        r#"
        SELECT balance
        FROM blackjack_bot_rust_users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?
    .balance;

    Ok(balance)
}

pub async fn create_table(pool: &Pool<Postgres>, channel_id: u64) -> Result<(), sqlx::Error> {
    let discord_channel_id = discord_id_to_i64(channel_id);

    sqlx::query!(
        r#"
        INSERT INTO blackjack_bot_rust_tables (discord_channel_id)
        VALUES ($1)
        "#,
        discord_channel_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_table(pool: &Pool<Postgres>, table_id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM blackjack_bot_rust_tables
        WHERE id = $1
        "#,
        table_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_table_id(pool: &Pool<Postgres>, channel_id: u64) -> Result<i32, sqlx::Error> {
    let channel_id = discord_id_to_i64(channel_id);

    let table_id = sqlx::query!(
        r#"
        SELECT id
        FROM blackjack_bot_rust_tables
        WHERE discord_channel_id = $1
        "#,
        channel_id
    )
    .fetch_one(pool)
    .await?
    .id;

    Ok(table_id)
}

pub async fn bet(pool: &Pool<Postgres>, user_id: UserId, amount: i32) -> Result<(), sqlx::Error> {
    let user_id = user_id.get_user_id(pool).await?;

    sqlx::query!(
        r#"
        UPDATE blackjack_bot_rust_users
        SET balance = balance - $1
        WHERE id = $2
        "#,
        amount,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn save_result(
    pool: &Pool<Postgres>,
    user_id: UserId,
    amount: i32,
) -> Result<(), sqlx::Error> {
    let user_id = user_id.get_user_id(pool).await?;

    sqlx::query!(
        r#"
        UPDATE blackjack_bot_rust_users
        SET balance = balance + $1
        WHERE id = $2
        "#,
        amount,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_bonus(pool: &Pool<Postgres>, amount: i32) -> Result<i32, sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO blackjack_bot_rust_bonuses (amount)
        VALUES ($1)
        "#,
        amount,
    )
    .execute(pool)
    .await?;

    let id = sqlx::query!(
        r#"
        SELECT id
        FROM blackjack_bot_rust_bonuses
        WHERE amount = $1
        "#,
        amount,
    )
    .fetch_one(pool)
    .await?
    .id;

    Ok(id)
}

pub async fn get_bonus(
    pool: &Pool<Postgres>,
    user_id: UserId,
    bonus_id: i32,
) -> Result<i32, sqlx::Error> {
    let user_id = user_id.get_user_id(pool).await?;

    sqlx::query!(
        r#"
        INSERT INTO blackjack_bot_rust_user_bonuses (user_id, bonus_id)
        VALUES ($1, $2)
        "#,
        user_id,
        bonus_id,
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        UPDATE blackjack_bot_rust_users
        SET balance = balance + (
            SELECT amount
            FROM blackjack_bot_rust_bonuses
            WHERE id = $1
        )
        WHERE id = $2
        "#,
        bonus_id,
        user_id,
    )
    .execute(pool)
    .await?;

    let amount = sqlx::query!(
        r#"
        SELECT amount
        FROM blackjack_bot_rust_bonuses
        WHERE id = $1
        "#,
        bonus_id,
    )
    .fetch_one(pool)
    .await?
    .amount;

    Ok(amount)
}
