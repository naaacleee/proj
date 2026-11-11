//! Диалог с наставником: обычный и потоковый (SSE) варианты.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use mentor_llm::{ChatMessage, ChatOptions};

use crate::auth_extract::auth_user;
use crate::error::ApiError;
use crate::state::AppState;

/// Сколько последних сообщений истории учитывается (пока память на сервере
/// не реализована в Фазе 1, историю передаёт клиент).
const MAX_HISTORY: usize = 20;

#[derive(Debug, Deserialize)]
pub struct HistoryItem {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequestBody {
    pub message: String,
    #[serde(default)]
    pub history: Vec<HistoryItem>,
}

#[derive(Debug, Serialize)]
pub struct ChatReply {
    pub reply: String,
    pub model: String,
}

fn build_messages(body: &ChatRequestBody) -> Result<Vec<ChatMessage>, ApiError> {
    let message = body.message.trim();
    if message.is_empty() {
        return Err(ApiError::BadRequest("message must not be empty".to_string()));
    }
    if message.chars().count() > 8000 {
        return Err(ApiError::BadRequest("message is too long".to_string()));
    }

    let mut messages = vec![ChatMessage {
        role: "system".to_string(),
        content: mentor_llm::persona::system_prompt().to_string(),
    }];

    for item in body.history.iter().rev().take(MAX_HISTORY).rev() {
        let role = match item.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => continue,
        };
        let content: String = item.content.chars().take(2000).collect();
        if content.trim().is_empty() {
            continue;
        }
        messages.push(ChatMessage {
            role: role.to_string(),
            content,
        });
    }

    messages.push(ChatMessage {
        role: "user".to_string(),
        content: message.to_string(),
    });
    Ok(messages)
}

pub async fn chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChatRequestBody>,
) -> Result<Json<ChatReply>, ApiError> {
    let _user = auth_user(&headers, &state)?;
    let messages = build_messages(&body)?;
    let reply = state
        .llm
        .complete(&messages, &ChatOptions::default())
        .await
        .map_err(ApiError::from)?;
    Ok(Json(ChatReply {
        reply,
        model: state.llm_model.clone(),
    }))
}

/// Потоковый ответ (SSE): события `{"delta": "..."}` по мере генерации,
/// в конце — `{"done": true}`. Ошибка модели приходит событием `{"error": ...}`.
pub async fn chat_stream(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChatRequestBody>,
) -> Result<Response, ApiError> {
    let _user = auth_user(&headers, &state)?;
    let messages = build_messages(&body)?;
    let stream = state
        .llm
        .stream(&messages, &ChatOptions::default())
        .await
        .map_err(ApiError::from)?;

    let events = stream.map(|item| {
        let data = match item {
            Ok(delta) => serde_json::json!({ "delta": delta }).to_string(),
            Err(err) => serde_json::json!({ "error": err.to_string() }).to_string(),
        };
        Ok::<Event, std::convert::Infallible>(Event::default().data(data))
    });
    // Всё, что попадает в SSE-ответ, должно жить без заимствований ('static),
    // поэтому имя модели клонируем заранее.
    let model_name = state.llm_model.clone();
    let events = futures_util::stream::once(async move {
        Ok::<Event, std::convert::Infallible>(
            Event::default().data(serde_json::json!({ "model": model_name }).to_string()),
        )
    })
    .chain(events)
    .chain(futures_util::stream::once(async {
        Ok::<Event, std::convert::Infallible>(
            Event::default().data(serde_json::json!({ "done": true }).to_string()),
        )
    }));

    Ok(Sse::new(events).keep_alive(KeepAlive::default()).into_response())
}
