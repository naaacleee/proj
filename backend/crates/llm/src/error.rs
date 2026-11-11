use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("llm http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("llm decode error: {0}")]
    Decode(#[from] serde_json::Error),

    #[error("llm api error: {0}")]
    Api(String),
}
