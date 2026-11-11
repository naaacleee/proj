use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// Ошибки HTTP-слоя с единым форматом ответа:
/// `{"error": {"code": "...", "message": "..."}}`.
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    Conflict(String),
    Upstream(String),
    Internal(String),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Upstream(_) => StatusCode::BAD_GATEWAY,
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            ApiError::BadRequest(_) => "bad_request",
            ApiError::Unauthorized => "unauthorized",
            ApiError::Conflict(_) => "conflict",
            ApiError::Upstream(_) => "upstream_error",
            ApiError::Internal(_) => "internal_error",
        }
    }

    fn message(&self) -> &str {
        match self {
            ApiError::BadRequest(m)
            | ApiError::Conflict(m)
            | ApiError::Upstream(m)
            | ApiError::Internal(m) => m,
            ApiError::Unauthorized => "invalid or missing access token",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({
            "error": { "code": self.code(), "message": self.message() }
        }));
        (self.status_code(), body).into_response()
    }
}

impl From<mentor_core::CoreError> for ApiError {
    fn from(err: mentor_core::CoreError) -> Self {
        use mentor_core::CoreError as C;
        match err {
            C::InvalidCredentials | C::InvalidToken => ApiError::Unauthorized,
            C::UsernameTaken => ApiError::Conflict("username already taken".to_string()),
            C::InvalidUsername(m) | C::WeakPassword(m) => ApiError::BadRequest(m),
            C::Hashing(m) | C::Internal(m) => ApiError::Internal(m),
            C::Db(db_err) => ApiError::Internal(format!("database error: {db_err}")),
        }
    }
}

impl From<mentor_llm::LlmError> for ApiError {
    fn from(err: mentor_llm::LlmError) -> Self {
        ApiError::Upstream(format!("model error: {err}"))
    }
}
