package com.mentor.core.network

import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import okhttp3.Call
import okhttp3.Callback
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import org.json.JSONObject
import java.io.IOException

/** Событие потокового ответа наставника. */
sealed interface SseEvent {
    /** Кусочек текста ответа. */
    data class Delta(val text: String) : SseEvent

    /** Ошибка (сеть или сервер). */
    data class Failed(val message: String) : SseEvent

    /** Генерация завершена. */
    data object Done : SseEvent
}

/**
 * Клиент потокового диалога (SSE) с наставником.
 *
 * Возвращает [Call] — его можно отменить (например, при уходе с экрана).
 */
class SseChatClient(private val client: OkHttpClient) {

    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
    }

    fun streamChat(
        baseUrl: String,
        token: String,
        message: String,
        history: List<ChatMessageDto>,
        onEvent: (SseEvent) -> Unit,
    ): Call {
        val payload = json.encodeToString(ChatRequestBody(message, history))
        val request = Request.Builder()
            .url(baseUrl.trimEnd('/') + "/api/v1/mentor/chat/stream")
            .header("Authorization", "Bearer $token")
            .header("Accept", "text/event-stream")
            .post(payload.toRequestBody("application/json".toMediaType()))
            .build()

        val call = client.newCall(request)
        call.enqueue(object : Callback {
            override fun onFailure(call: Call, e: IOException) {
                if (!call.isCanceled()) {
                    onEvent(SseEvent.Failed(e.message ?: "network error"))
                }
            }

            override fun onResponse(call: Call, response: Response) {
                response.use { resp ->
                    if (!resp.isSuccessful) {
                        onEvent(SseEvent.Failed("server error: HTTP ${resp.code}"))
                        return
                    }
                    val source = resp.body?.source()
                    if (source == null) {
                        onEvent(SseEvent.Failed("empty response"))
                        return
                    }
                    try {
                        var finished = false
                        while (!finished && !call.isCanceled()) {
                            val line = source.readUtf8Line() ?: break
                            if (!line.startsWith("data:")) continue
                            val data = line.removePrefix("data:").trim()
                            if (data.isEmpty()) continue
                            finished = handleData(data, onEvent)
                        }
                        if (!call.isCanceled()) {
                            onEvent(SseEvent.Done)
                        }
                    } catch (e: IOException) {
                        if (!call.isCanceled()) {
                            onEvent(SseEvent.Failed(e.message ?: "stream error"))
                        }
                    }
                }
            }
        })
        return call
    }

    /** @return true, если поток завершён. */
    private fun handleData(data: String, onEvent: (SseEvent) -> Unit): Boolean {
        val obj = try {
            JSONObject(data)
        } catch (_: Exception) {
            return false
        }
        when {
            obj.has("delta") -> onEvent(SseEvent.Delta(obj.optString("delta", "")))
            obj.has("error") -> onEvent(SseEvent.Failed(obj.optString("error", "model error")))
            obj.has("done") -> return true
            // событие {"model": ...} — информационное, игнорируем
        }
        return false
    }
}
