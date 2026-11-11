use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("username already taken")]
    UsernameTaken,

    #[error("invalid username: {0}")]
    InvalidUsername(String),

    #[error("weak password: {0}")]
    WeakPassword(String),

    #[error("invalid or expired token")]
    InvalidToken,

    #[error("hashing error: {0}")]
    Hashing(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Db(#[from] mentor_db::DbError),
}
