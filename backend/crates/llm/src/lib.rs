//! Слой абстракции над языковыми моделями.
//!
//! Все обращения к модели идут через [`LlmProvider`]: рантайм (llama.cpp,
//! Ollama) подключается по OpenAI-совместимому HTTP API. Это позволяет
//! менять модели и рантаймы без изменений в домене.

pub mod error;
pub mod persona;
pub mod provider;

pub use error::LlmError;
pub use provider::{BoxStream, BoxStreamResult, ChatMessage, ChatOptions, LlmProvider, OpenAiCompatible};
