use dotenvy::dotenv;
use sqlx::{postgres::PgPoolOptions, Executor, Pool, Postgres};
use std::env;

use crate::db::user_id::UserId;

pub async fn establish_connection() -> Result<Pool<Postgres>, sqlx::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

pub async fn get_name_from_db(
    conn: &Pool<Postgres>,
    user_id: UserId,
) -> Result<String, sqlx::Error> {
    let user_id = user_id.get_from_db(conn).await?;

    let record = sqlx::query!(
        r#"
        SELECT name
        FROM blackjack_bot_rust_users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(conn)
    .await?;

    Ok(record.name)
}

pub async fn get_name(user_id: UserId) -> Result<String, sqlx::Error> {
    let conn = establish_connection().await?;
    let name = get_name_from_db(&conn, user_id).await?;

    Ok(name)
}

pub async fn register(id: UserId, name: String) -> Result<String, sqlx::Error> {
    let conn = establish_connection().await?;

    let result = conn
        .execute(sqlx::query!(
            r#"
        INSERT INTO blackjack_bot_rust_users (name, balance)
        VALUES ($1, 0)
        "#,
            name
        ))
        .await;
    if let UserId::Cli(_) = id {
        return result.map(|_| name);
    }

    if let UserId::Discord(id) = id {
        let user_id = UserId::Name(name.clone()).get_from_db(&conn).await?;
        let discord_id = id;

        conn.execute(sqlx::query!(
            r#"
            INSERT INTO blackjack_bot_rust_discord_users (user_id, discord_id)
            VALUES ($1, $2)
            "#,
            user_id,
            discord_id
        ))
        .await?;
    }

    Ok(name)
}

pub async fn bet(user_id: UserId, amount: i32) -> Result<String, sqlx::Error> {
    let conn = establish_connection().await?;
    let id = user_id.get_from_db(&conn).await?;

    conn.execute(sqlx::query!(
        r#"
        UPDATE blackjack_bot_rust_users
        SET balance = balance - $1
        WHERE id = $2
        "#,
        amount,
        id
    ))
    .await?;

    let name = get_name_from_db(&conn, user_id).await?;

    Ok(name)
}

pub async fn rename(id: UserId, new_name: String) -> Result<(String, String), sqlx::Error> {
    let conn = establish_connection().await?;
    let old_name = get_name_from_db(&conn, id.clone()).await?;
    let id = id.get_from_db(&conn).await?;

    conn.execute(sqlx::query!(
        r#"
        UPDATE blackjack_bot_rust_users
        SET name = $1
        WHERE id = $2
        "#,
        new_name,
        id
    ))
    .await?;

    Ok((old_name, new_name))
}

pub async fn balance(id: UserId) -> Result<(String, i32), sqlx::Error> {
    let conn = establish_connection().await?;
    let name = get_name_from_db(&conn, id.clone()).await?;
    let id = id.get_from_db(&conn).await?;

    let record = sqlx::query!(
        r#"
        SELECT balance
        FROM blackjack_bot_rust_users
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(&conn)
    .await?;

    Ok((name, record.balance))
}

pub async fn bonus(id: UserId, bonus_id: i32) -> Result<(String, i32), sqlx::Error> {
    let conn = establish_connection().await?;
    let name = get_name_from_db(&conn, id.clone()).await?;
    let id = id.get_from_db(&conn).await?;

    let mut tx = conn.begin().await?;

    tx.execute(sqlx::query!(
        r#"
        INSERT INTO blackjack_bot_rust_user_bonuses (user_id, bonus_id)
        VALUES ($1, $2)
        "#,
        id,
        bonus_id
    ))
    .await?;

    tx.execute(sqlx::query!(
        r#"
        UPDATE blackjack_bot_rust_users
        SET balance = balance + (SELECT amount FROM blackjack_bot_rust_bonuses WHERE id = $1)
        WHERE id = $2
        "#,
        bonus_id,
        id
    ))
    .await?;

    let bonus_amount = sqlx::query!(
        r#"
        SELECT amount
        FROM blackjack_bot_rust_bonuses
        WHERE id = $1
        "#,
        bonus_id
    )
    .fetch_one(&mut *tx)
    .await?
    .amount;

    tx.commit().await?;

    Ok((name, bonus_amount))
}

pub async fn refund(id: UserId, amount: i32) -> Result<String, sqlx::Error> {
    let conn = establish_connection().await?;
    let name = get_name_from_db(&conn, id.clone()).await?;
    let id = id.get_from_db(&conn).await?;

    conn.execute(sqlx::query!(
        r#"
        UPDATE blackjack_bot_rust_users
        SET balance = balance + $1
        WHERE id = $2
        "#,
        amount,
        id
    ))
    .await?;

    Ok(name)
}
