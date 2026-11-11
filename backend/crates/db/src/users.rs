//! Репозиторий пользователей.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::DbError;
use crate::Pool;

/// Строка пользователя.
///
/// `password_hash` намеренно присутствует только на сервере и никогда
/// не сериализуется в API-ответы.
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create(pool: &Pool, username: &str, password_hash: &str) -> Result<UserRow, DbError> {
    let row = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (id, username, password_hash) \
         VALUES ($1, $2, $3) \
         RETURNING id, username, password_hash, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await
    .map_err(map_unique_violation)?;
    Ok(row)
}

pub async fn find_by_username(pool: &Pool, username: &str) -> Result<Option<UserRow>, DbError> {
    Ok(sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_by_id(pool: &Pool, id: Uuid) -> Result<Option<UserRow>, DbError> {
    Ok(sqlx::query_as::<_, UserRow>(
        "SELECT id, username, password_hash, created_at FROM users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

/// Нарушение уникальности (23505) превращаем в отдельный вариант ошибки.
fn map_unique_violation(err: sqlx::Error) -> DbError {
    if let sqlx::Error::Database(ref db_err) = err {
        if db_err.code().as_deref() == Some("23505") {
            return DbError::Conflict;
        }
    }
    DbError::Sqlx(err)
}
