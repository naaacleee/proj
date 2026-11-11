//! mentor-api — HTTP-сервер персонального ИИ-наставника.
//!
//! Модульный монолит: авторизация, диалог с наставником (локальная LLM),
//! healthchecks. Остальные подсистемы (память, цели, расписание) — Фаза 1.

use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use mentor_core::auth::{AuthConfig, AuthService};
use mentor_llm::{LlmProvider, OpenAiCompatible};

mod auth_extract;
mod config;
mod error;
mod routes;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let cfg = Config::from_env()?;
    tracing::info!(bind = %cfg.bind, llm = %cfg.llm.base_url, "starting mentor-api");

    let pool = mentor_db::connect(&cfg.database_url).await?;
    mentor_db::migrate(&pool).await?;

    let llm: Arc<dyn LlmProvider> = Arc::new(OpenAiCompatible::new(
        &cfg.llm.base_url,
        cfg.llm.api_key.clone(),
        cfg.llm.chat_model.clone(),
        cfg.llm.embed_model.clone(),
    ));

    let state = AppState {
        db: pool.clone(),
        auth: Arc::new(AuthService::new(
            AuthConfig {
                jwt_secret: cfg.jwt_secret.clone(),
                access_ttl_secs: cfg.access_ttl_secs,
                refresh_ttl_days: cfg.refresh_ttl_days,
            },
            pool,
        )),
        llm,
        llm_model: cfg.llm.chat_model.clone(),
    };

    let app = build_app(state, &cfg);

    let listener = tokio::net::TcpListener::bind(&cfg.bind).await?;
    tracing::info!("listening on http://{}", cfg.bind);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn build_app(state: AppState, cfg: &Config) -> Router {
    let cors = if cfg.cors_allow_origin == "*" {
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(
                cfg.cors_allow_origin
                    .parse::<HeaderValue>()
                    .expect("CORS_ALLOW_ORIGIN must be a valid header value"),
            )
            .allow_methods([Method::GET, Method::POST])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
            ])
    };

    routes::router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,tower_http=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

async fn shutdown_signal() {
    if tokio::signal::ctrl_c().await.is_ok() {
        tracing::info!("received ctrl-c, shutting down");
    }
}
