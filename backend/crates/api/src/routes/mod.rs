use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::auth_extract::auth_user;
use crate::error::ApiError;
use crate::state::AppState;

pub mod auth;
pub mod mentor;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/me", get(me))
        .nest("/api/v1/auth", auth::router())
        .route("/api/v1/mentor/chat", post(mentor::chat))
        .route("/api/v1/mentor/chat/stream", post(mentor::chat_stream))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "ready", "db": "ok" })),
        ),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "not_ready", "db": err.to_string() })),
        ),
    }
}

async fn me(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<auth::UserDto>, ApiError> {
    let user = auth_user(&headers, &state)?;
    Ok(Json(auth::UserDto {
        id: user.id.to_string(),
        username: user.username,
    }))
}
