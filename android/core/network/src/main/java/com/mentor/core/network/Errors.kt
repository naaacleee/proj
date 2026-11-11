package com.mentor.core.network

import kotlinx.serialization.json.Json
import retrofit2.HttpException

/** Человекочитаемое описание ошибки API. */
fun describeError(t: Throwable): String {
    if (t is HttpException) {
        val body = t.response()?.errorBody()?.string()
        if (!body.isNullOrBlank()) {
            try {
                val parsed = Json { ignoreUnknownKeys = true }
                    .decodeFromString(ErrorBody.serializer(), body)
                return parsed.error.message
            } catch (_: Exception) {
                // формат ответа неожиданный — покажем код
            }
        }
        return "Ошибка сервера: ${t.code()}"
    }
    return t.message ?: "Неизвестная ошибка"
}
