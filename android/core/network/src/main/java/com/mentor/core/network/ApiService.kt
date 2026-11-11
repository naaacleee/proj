package com.mentor.core.network

import kotlinx.serialization.Serializable
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST

// ---- DTO (имена полей в snake_case совпадают с API бэкенда) ----

@Serializable
data class UserDto(val id: String, val username: String)

@Serializable
data class AuthResponse(
    val access_token: String,
    val token_type: String,
    val expires_in: Long,
    val refresh_token: String,
    val user: UserDto,
)

@Serializable
data class RegisterRequest(val username: String, val password: String)

@Serializable
data class LoginRequest(val username: String, val password: String)

@Serializable
data class RefreshRequest(val refresh_token: String)

@Serializable
data class StatusResponse(val status: String)

@Serializable
data class ChatMessageDto(val role: String, val content: String)

@Serializable
data class ChatRequestBody(val message: String, val history: List<ChatMessageDto> = emptyList())

@Serializable
data class ChatReply(val reply: String, val model: String)

@Serializable
data class ErrorDetail(val code: String, val message: String)

@Serializable
data class ErrorBody(val error: ErrorDetail)

// ---- REST API ----

interface ApiService {
    @POST("api/v1/auth/register")
    suspend fun register(@Body body: RegisterRequest): AuthResponse

    @POST("api/v1/auth/login")
    suspend fun login(@Body body: LoginRequest): AuthResponse

    @POST("api/v1/auth/refresh")
    suspend fun refresh(@Body body: RefreshRequest): AuthResponse

    @POST("api/v1/auth/logout")
    suspend fun logout(@Body body: RefreshRequest): StatusResponse

    @GET("api/v1/me")
    suspend fun me(): UserDto

    @POST("api/v1/mentor/chat")
    suspend fun chat(@Body body: ChatRequestBody): ChatReply
}
