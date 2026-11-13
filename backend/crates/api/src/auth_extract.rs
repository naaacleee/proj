//! Извлечение пользователя из access-токена в заголовке Authorization.

use axum::http::{header::AUTHORIZATION, HeaderMap};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
}

pub fn auth_user(headers: &HeaderMap, state: &AppState) -> Result<AuthUser, ApiError> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;
    let token = value
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized)?;
    let claims = state.auth.verify_access(token)?;
    let id = Uuid::parse_str(&claims.sub).map_err(|_| ApiError::Unauthorized)?;
    Ok(AuthUser {
        id,
        username: claims.username,
    })
}
