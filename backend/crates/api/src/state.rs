use std::sync::Arc;

use mentor_core::auth::AuthService;
use mentor_db::Pool;
use mentor_llm::LlmProvider;

/// Общее состояние приложения для обработчиков.
#[derive(Clone)]
pub struct AppState {
    pub db: Pool,
    pub auth: Arc<AuthService>,
    pub llm: Arc<dyn LlmProvider>,
    /// Имя активной модели (для ответов API).
    pub llm_model: String,
}
