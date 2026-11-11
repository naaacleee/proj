//! Репозиторий refresh-токенов.
//!
//! В базе хранится только SHA-256 хеш токена — сам токен живёт лишь
//! у клиента, поэтому утечка базы не даёт действующих сессий.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::DbError;
use crate::Pool;

#[derive(Debug, Clone, FromRow)]
pub struct RefreshTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn insert(
    pool: &Pool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Найти действующий токен: не отозван и не истёк.
pub async fn find_valid(
    pool: &Pool,
    token_hash: &str,
    now: DateTime<Utc>,
) -> Result<Option<RefreshTokenRow>, DbError> {
    Ok(sqlx::query_as::<_, RefreshTokenRow>(
        "SELECT id, user_id, token_hash, expires_at, revoked_at, created_at \
         FROM refresh_tokens \
         WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > $2",
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await?)
}

/// Отозвать токен (идемпотентно).
pub async fn revoke(pool: &Pool, token_hash: &str) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now() \
         WHERE token_hash = $1 AND revoked_at IS NULL",
    )
    .bind(token_hash)
    .execute(pool)
    .await?;
    Ok(())
}
