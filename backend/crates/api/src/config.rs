use std::env;

/// Конфигурация приложения (переменные окружения + `.env`).
#[derive(Debug, Clone)]
pub struct Config {
    /// Окружение: `dev` или `production`.
    pub env: String,
    pub database_url: String,
    pub bind: String,
    pub jwt_secret: String,
    pub access_ttl_secs: i64,
    pub refresh_ttl_days: i64,
    pub llm: LlmConfig,
    pub cors_allow_origin: String,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Базовый URL локального рантайма моделей (llama.cpp, Ollama).
    pub base_url: String,
    pub api_key: Option<String>,
    pub chat_model: String,
    pub embed_model: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let env_name = env_or("MENTOR_ENV", "dev");
        let is_production = env_name == "production";

        let jwt_secret = env_or(
            "MENTOR_JWT_SECRET",
            "dev-only-secret-change-me-please-0123456789",
        );
        if jwt_secret.len() < 32 {
            anyhow::bail!("MENTOR_JWT_SECRET must be at least 32 characters");
        }
        if is_production && jwt_secret.starts_with("dev-only") {
            anyhow::bail!(
                "MENTOR_ENV=production requires a real MENTOR_JWT_SECRET \
                 (generate one: openssl rand -base64 48)"
            );
        }
        if !is_production && jwt_secret.starts_with("dev-only") {
            tracing::warn!(
                "MENTOR_JWT_SECRET uses the development default — set a real secret before production"
            );
        }

        let cors_allow_origin = env_or("CORS_ALLOW_ORIGIN", "*");
        if is_production && cors_allow_origin == "*" {
            tracing::warn!(
                "CORS_ALLOW_ORIGIN='*' in production: fine for a native mobile client, \
                 restrict it if you add a web version"
            );
        }

        Ok(Self {
            env: env_name,
            database_url: env_or(
                "DATABASE_URL",
                "postgres://mentor:mentor@127.0.0.1:5432/mentor",
            ),
            bind: env_or("MENTOR_BIND", "0.0.0.0:8080"),
            jwt_secret,
            access_ttl_secs: env_parse("MENTOR_ACCESS_TTL_SECS", 900)?,
            refresh_ttl_days: env_parse("MENTOR_REFRESH_TTL_DAYS", 30)?,
            llm: LlmConfig {
                base_url: env_or("LLM_BASE_URL", "http://127.0.0.1:8081"),
                api_key: env::var("LLM_API_KEY").ok().filter(|s| !s.is_empty()),
                chat_model: env_or("LLM_CHAT_MODEL", "qwen3:4b"),
                embed_model: env_or("LLM_EMBED_MODEL", "bge-m3"),
            },
            cors_allow_origin,
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> anyhow::Result<T> {
    match env::var(key) {
        Ok(v) if !v.trim().is_empty() => v
            .trim()
            .parse::<T>()
            .map_err(|_| anyhow::anyhow!("invalid value for {key}: {v}")),
        _ => Ok(default),
    }
}
