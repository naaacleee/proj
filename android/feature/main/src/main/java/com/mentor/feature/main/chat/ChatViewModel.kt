package com.mentor.feature.main.chat

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import com.mentor.core.network.ApiService
import com.mentor.core.network.ChatMessageDto
import com.mentor.core.network.SseChatClient
import com.mentor.core.network.SseEvent
import com.mentor.core.network.TokenStore
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import okhttp3.Call

data class ChatMessageUi(val role: String, val content: String)

class ChatViewModel(
    @Suppress("unused") private val api: ApiService,
    private val tokenStore: TokenStore,
    private val sseClient: SseChatClient,
    private val baseUrlProvider: () -> String,
) : ViewModel() {

    private val _messages = MutableStateFlow<List<ChatMessageUi>>(emptyList())
    val messages: StateFlow<List<ChatMessageUi>> = _messages

    private val _sending = MutableStateFlow(false)
    val sending: StateFlow<Boolean> = _sending

    private var activeCall: Call? = null

    fun send(text: String) {
        val message = text.trim()
        if (message.isEmpty() || _sending.value) return

        val token = tokenStore.accessToken
        if (token.isNullOrBlank()) {
            _messages.update { it + ChatMessageUi("assistant", "Сессия истекла — войдите заново.") }
            return
        }

        // Историю диалога пока держит клиент (серверная память — Фаза 1).
        val history = _messages.value
            .filter { it.content.isNotBlank() }
            .takeLast(20)
            .map { ChatMessageDto(it.role, it.content) }

        _messages.update {
            it + ChatMessageUi("user", message) + ChatMessageUi("assistant", "")
        }
        _sending.value = true

        activeCall = sseClient.streamChat(baseUrlProvider(), token, message, history) { event ->
            when (event) {
                is SseEvent.Delta -> appendToLast(event.text)
                is SseEvent.Failed -> {
                    appendToLast("⚠ ${event.message}")
                    _sending.value = false
                }
                SseEvent.Done -> _sending.value = false
            }
        }
    }

    private fun appendToLast(text: String) {
        _messages.update { list ->
            if (list.isEmpty()) {
                list
            } else {
                list.dropLast(1) + list.last().copy(content = list.last().content + text)
            }
        }
    }

    override fun onCleared() {
        activeCall?.cancel()
        super.onCleared()
    }
}

class ChatViewModelFactory(
    private val api: ApiService,
    private val tokenStore: TokenStore,
    private val sseClient: SseChatClient,
    private val baseUrlProvider: () -> String,
) : ViewModelProvider.Factory {
    @Suppress("UNCHECKED_CAST")
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(ChatViewModel::class.java)) {
            return ChatViewModel(api, tokenStore, sseClient, baseUrlProvider) as T
        }
        throw IllegalArgumentException("unknown ViewModel: ${modelClass.name}")
    }
}
