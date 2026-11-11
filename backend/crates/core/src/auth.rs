//! Регистрация, вход, JWT-доступ и ротация refresh-токенов.
//!
//! Схема токенов:
//! - access-токен — короткоживущий JWT (HS256), не хранится на сервере;
//! - refresh-токен — случайная строка, в базе лежит только её SHA-256 хеш;
//!   при каждом обновлении старый токен отзывается (ротация).

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use mentor_db::users::UserRow;
use mentor_db::Pool;

use crate::error::CoreError;

// ---- Конфигурация и публичные модели ----

#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Секрет для подписи JWT (минимум 32 символа).
    pub jwt_secret: String,
    /// Время жизни access-токена, секунды.
    pub access_ttl_secs: i64,
    /// Время жизни refresh-токена, дни.
    pub refresh_ttl_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Идентификатор пользователя (UUID строкой).
    pub sub: String,
    pub username: String,
    pub iat: u64,
    pub exp: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

// ---- Валидация ----

/// Имя пользователя: 3–32 символа, латиница/цифры/`_`/`-`/`.`.
pub fn validate_username(username: &str) -> Result<(), CoreError> {
    let len = username.chars().count();
    if !(3..=32).contains(&len) {
        return Err(CoreError::InvalidUsername(
            "must be 3-32 characters long".into(),
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(CoreError::InvalidUsername(
            "only latin letters, digits, '_', '-' and '.' are allowed".into(),
        ));
    }
    Ok(())
}

/// Пароль: 8–128 символов.
pub fn validate_password(password: &str) -> Result<(), CoreError> {
    let len = password.chars().count();
    if !(8..=128).contains(&len) {
        return Err(CoreError::WeakPassword(
            "must be 8-128 characters long".into(),
        ));
    }
    Ok(())
}

// ---- Пароли (argon2id) ----

pub fn hash_password(password: &str) -> Result<String, CoreError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| CoreError::Hashing(err.to_string()))
}

/// Проверка пароля. Любая ошибка разбора хеша — «не совпало».
pub fn verify_password(password: &str, password_hash: &str) -> bool {
    PasswordHash::new(password_hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

// ---- Сервис ----

pub struct AuthService {
    cfg: AuthConfig,
    pool: Pool,
}

impl AuthService {
    pub fn new(cfg: AuthConfig, pool: Pool) -> Self {
        Self { cfg, pool }
    }

    pub async fn register(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(UserRow, TokenPair), CoreError> {
        validate_username(username)?;
        validate_password(password)?;
        let password_hash = hash_password(password)?;
        let user = mentor_db::users::create(&self.pool, username, &password_hash)
            .await
            .map_err(|err| match err {
                mentor_db::DbError::Conflict => CoreError::UsernameTaken,
                other => CoreError::Db(other),
            })?;
        let tokens = self.issue_tokens(&user).await?;
        Ok((user, tokens))
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(UserRow, TokenPair), CoreError> {
        let user = mentor_db::users::find_by_username(&self.pool, username)
            .await?
            .ok_or(CoreError::InvalidCredentials)?;
        if !verify_password(password, &user.password_hash) {
            return Err(CoreError::InvalidCredentials);
        }
        let tokens = self.issue_tokens(&user).await?;
        Ok((user, tokens))
    }

    /// Обновление пары токенов со старой ротации: старый refresh отзывается.
    pub async fn refresh(&self, refresh_token: &str) -> Result<(UserRow, TokenPair), CoreError> {
        let hash = sha256_hex(refresh_token.as_bytes());
        let row = mentor_db::tokens::find_valid(&self.pool, &hash, Utc::now())
            .await?
            .ok_or(CoreError::InvalidToken)?;
        mentor_db::tokens::revoke(&self.pool, &hash).await?;
        let user = mentor_db::users::find_by_id(&self.pool, row.user_id)
            .await?
            .ok_or(CoreError::InvalidToken)?;
        let tokens = self.issue_tokens(&user).await?;
        Ok((user, tokens))
    }

    /// Выход: отзыв текущего refresh-токена (идемпотентно).
    pub async fn logout(&self, refresh_token: &str) -> Result<(), CoreError> {
        let hash = sha256_hex(refresh_token.as_bytes());
        mentor_db::tokens::revoke(&self.pool, &hash).await?;
        Ok(())
    }

    /// Проверка access-токена и извлечение утверждений.
    pub fn verify_access(&self, token: &str) -> Result<Claims, CoreError> {
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.cfg.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|_| CoreError::InvalidToken)?;
        Ok(data.claims)
    }

    // ---- внутреннее ----

    async fn issue_tokens(&self, user: &UserRow) -> Result<TokenPair, CoreError> {
        let access_token = self.issue_access(user)?;
        let refresh_token = new_refresh_token();
        let hash = sha256_hex(refresh_token.as_bytes());
        let expires_at = Utc::now() + Duration::days(self.cfg.refresh_ttl_days);
        mentor_db::tokens::insert(&self.pool, user.id, &hash, expires_at).await?;
        Ok(TokenPair {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: self.cfg.access_ttl_secs,
            refresh_token,
        })
    }

    fn issue_access(&self, user: &UserRow) -> Result<String, CoreError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            iat: now,
            exp: now.saturating_add(self.cfg.access_ttl_secs as u64),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.cfg.jwt_secret.as_bytes()),
        )
        .map_err(|err| CoreError::Internal(format!("jwt encode: {err}")))
    }
}

// ---- утилиты ----

fn sha256_hex(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

/// Случайный refresh-токен: 64 hex-символа (два UUIDv4).
fn new_refresh_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> AuthService {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://mentor:mentor@127.0.0.1:5432/mentor")
            .expect("lazy pool");
        AuthService::new(
            AuthConfig {
                jwt_secret: "unit-test-secret-key-0123456789abcdef".to_string(),
                access_ttl_secs: 900,
                refresh_ttl_days: 30,
            },
            pool,
        )
    }

    #[test]
    fn password_hash_roundtrip() {
        let hash = hash_password("supersecret123").expect("hash");
        assert!(verify_password("supersecret123", &hash));
        assert!(!verify_password("wrongpassword", &hash));
        assert!(!verify_password("supersecret123", "not-a-valid-hash"));
    }

    #[test]
    fn username_validation_rules() {
        assert!(validate_username("alice_01").is_ok());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("bad space").is_err());
        assert!(validate_username("кириллица").is_err());
        assert!(validate_username(&"x".repeat(33)).is_err());
    }

    #[test]
    fn password_validation_rules() {
        assert!(validate_password("longenough").is_ok());
        assert!(validate_password("short").is_err());
    }

    #[test]
    fn jwt_roundtrip() {
        let svc = service();
        let user = UserRow {
            id: Uuid::new_v4(),
            username: "alice".to_string(),
            password_hash: String::new(),
            created_at: Utc::now(),
        };
        let token = svc.issue_access(&user).expect("issue");
        let claims = svc.verify_access(&token).expect("verify");
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.sub, user.id.to_string());
    }

    #[test]
    fn jwt_rejects_garbage() {
        let svc = service();
        assert!(svc.verify_access("not-a-jwt").is_err());
    }
}
