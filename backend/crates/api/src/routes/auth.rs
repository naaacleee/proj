//! Эндпоинты регистрации, входа и работы с токенами.

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, State};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use mentor_core::auth::TokenPair;
use mentor_db::users::UserRow;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub user: UserDto,
}

impl AuthResponse {
    fn from_parts(user: UserRow, tokens: TokenPair) -> Self {
        Self {
            access_token: tokens.access_token,
            token_type: tokens.token_type,
            expires_in: tokens.expires_in,
            refresh_token: tokens.refresh_token,
            user: UserDto {
                id: user.id.to_string(),
                username: user.username,
            },
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    if !state.rate_limiter.allow(&format!("register:{}", addr.ip())) {
        return Err(ApiError::TooManyRequests(
            "too many attempts, try again later".to_string(),
        ));
    }
    let (user, tokens) = state
        .auth
        .register(req.username.trim(), &req.password)
        .await?;
    Ok(Json(AuthResponse::from_parts(user, tokens)))
}

async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    if !state.rate_limiter.allow(&format!("login:{}", addr.ip())) {
        return Err(ApiError::TooManyRequests(
            "too many attempts, try again later".to_string(),
        ));
    }
    let (user, tokens) = state
        .auth
        .login(req.username.trim(), &req.password)
        .await?;
    Ok(Json(AuthResponse::from_parts(user, tokens)))
}

async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let (user, tokens) = state.auth.refresh(&req.refresh_token).await?;
    Ok(Json(AuthResponse::from_parts(user, tokens)))
}

async fn logout(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.auth.logout(&req.refresh_token).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
