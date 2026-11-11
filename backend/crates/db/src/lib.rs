//! Доступ к данным: PostgreSQL как единый source of truth.

pub mod error;
pub mod tokens;
pub mod users;

pub use error::DbError;

use sqlx::postgres::PgPoolOptions;

pub type Pool = sqlx::PgPool;

/// Подключиться к базе данных.
pub async fn connect(database_url: &str) -> Result<Pool, DbError> {
    Ok(PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?)
}

/// Применить миграции (удобно для разработки; в проде — sqlx-cli).
pub async fn migrate(pool: &Pool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
