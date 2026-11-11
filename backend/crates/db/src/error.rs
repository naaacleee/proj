use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// Нарушение уникальности (например, занятое имя пользователя).
    #[error("conflict")]
    Conflict,
}
