use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::error::LlmError;

pub use futures_util::stream::BoxStream;

/// Поток дельт текста (кусочки ответа модели по мере генерации).
pub type BoxStreamResult = BoxStream<'static, Result<String, LlmError>>;

/// Одно сообщение диалога (формат OpenAI-совместимого API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Параметры генерации.
#[derive(Debug, Clone)]
pub struct ChatOptions {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            max_tokens: Some(512),
            temperature: Some(0.7),
        }
    }
}

/// Абстракция над языковой моделью.
///
/// Реализация [`OpenAiCompatible`] работает с любым рантаймом,
/// предоставляющим OpenAI-совместимый API (llama.cpp, Ollama и др.).
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Негенеративный запрос: вернуть полный текст ответа.
    async fn complete(&self, messages: &[ChatMessage], opts: &ChatOptions) -> Result<String, LlmError>;

    /// Генерация потоком: вернуть дельты текста по мере генерации.
    async fn stream(&self, messages: &[ChatMessage], opts: &ChatOptions) -> Result<BoxStreamResult, LlmError>;

    /// Эмбеддинги текстов (для семантической памяти).
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LlmError>;
}

/// Клиент OpenAI-совместимого API (llama.cpp, Ollama, ...).
///
/// Примечание: клиент собран без TLS — Phase 0 предполагает обычный HTTP
/// внутри частной сети (docker-compose). TLS терминируется на reverse-proxy
/// при развёртывании на VPS, поэтому URL рантайма должен начинаться с `http://`.
pub struct OpenAiCompatible {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    chat_model: String,
    embed_model: String,
}

impl OpenAiCompatible {
    pub fn new(
        base_url: &str,
        api_key: Option<String>,
        chat_model: String,
        embed_model: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            chat_model,
            embed_model,
        }
    }

    fn chat_body(&self, messages: &[ChatMessage], opts: &ChatOptions, stream: bool) -> serde_json::Value {
        serde_json::json!({
            "model": self.chat_model,
            "messages": messages,
            "temperature": opts.temperature.unwrap_or(0.7),
            "max_tokens": opts.max_tokens.unwrap_or(512),
            "stream": stream,
        })
    }

    fn authorize(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(key) => req.bearer_auth(key),
            None => req,
        }
    }
}

// ---- Модели ответов OpenAI-совместимого API ----

#[derive(Debug, Deserialize)]
struct ChatCompletion {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: StreamDelta,
}

#[derive(Debug, Deserialize, Default)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingsResponse {
    data: Vec<EmbeddingItem>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingItem {
    index: usize,
    embedding: Vec<f32>,
}

#[async_trait]
impl LlmProvider for OpenAiCompatible {
    async fn complete(&self, messages: &[ChatMessage], opts: &ChatOptions) -> Result<String, LlmError> {
        let body = self.chat_body(messages, opts, false);
        let req = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body);
        let resp = self.authorize(req).send().await?.error_for_status()?;
        let parsed: ChatCompletion = resp.json().await?;
        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| LlmError::Api("empty choices in chat completion".into()))
    }

    async fn stream(&self, messages: &[ChatMessage], opts: &ChatOptions) -> Result<BoxStreamResult, LlmError> {
        let body = self.chat_body(messages, opts, true);
        let req = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .json(&body);
        let resp = self.authorize(req).send().await?.error_for_status()?;
        Ok(sse_content_stream(resp))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LlmError> {
        let body = serde_json::json!({
            "model": self.embed_model,
            "input": texts,
        });
        let req = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .json(&body);
        let resp = self.authorize(req).send().await?.error_for_status()?;
        let mut parsed: EmbeddingsResponse = resp.json().await?;
        parsed.data.sort_by_key(|item| item.index);
        Ok(parsed.data.into_iter().map(|item| item.embedding).collect())
    }
}

// ---- Разбор SSE-потока OpenAI-совместимого API ----

/// Поток `data: {...}` строк с дельтами текста; завершение — `data: [DONE]`.
fn sse_content_stream(resp: reqwest::Response) -> BoxStreamResult {
    // Куски байтов переводим в Vec<u8>, чтобы не ссылаться на тип `Bytes`
    // изнутри reqwest в состоянии разборщика.
    let byte_stream = resp.bytes_stream().map(|chunk| chunk.map(|bytes| bytes.to_vec()));
    let initial = SseState {
        inner: Box::pin(byte_stream),
        buf: String::new(),
        done: false,
    };
    let stream = futures_util::stream::unfold(initial, |mut st| async move {
        loop {
            // Если в буфере есть полная строка — обрабатываем её.
            if let Some(pos) = st.buf.find('\n') {
                let mut line: String = st.buf.drain(..=pos).collect();
                while line.ends_with('\n') || line.ends_with('\r') {
                    line.pop();
                }
                if let Some(data) = line.strip_prefix("data:") {
                    let data = data.trim();
                    if data == "[DONE]" {
                        return None;
                    }
                    if data.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<StreamChunk>(data) {
                        Ok(chunk) => {
                            let delta = chunk
                                .choices
                                .into_iter()
                                .next()
                                .and_then(|c| c.delta.content)
                                .unwrap_or_default();
                            if delta.is_empty() {
                                continue;
                            }
                            return Some((Ok(delta), st));
                        }
                        Err(err) => return Some((Err(LlmError::Decode(err)), st)),
                    }
                }
                // Комментарий или пустая строка SSE — пропускаем.
                continue;
            }
            if st.done {
                return None;
            }
            // Пытаемся дочитать следующий кусок ответа.
            match st.inner.next().await {
                Some(Ok(bytes)) => {
                    st.buf.push_str(&String::from_utf8_lossy(&bytes));
                }
                Some(Err(err)) => {
                    st.done = true;
                    return Some((Err(LlmError::Http(err)), st));
                }
                None => {
                    st.done = true;
                }
            }
        }
    });
    Box::pin(stream)
}

struct SseState {
    inner: BoxStream<'static, Result<Vec<u8>, reqwest::Error>>,
    buf: String,
    done: bool,
}
