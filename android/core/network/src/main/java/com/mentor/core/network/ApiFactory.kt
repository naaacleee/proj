package com.mentor.core.network

import com.jakewharton.retrofit2.converter.kotlinx.serialization.asConverterFactory
import kotlinx.serialization.json.Json
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Response
import retrofit2.Retrofit
import java.util.concurrent.TimeUnit

/** Фабрика HTTP-клиента и Retrofit-сервиса. */
object ApiFactory {

    private val json = Json {
        ignoreUnknownKeys = true
        coerceInputValues = true
    }

    fun createClient(tokenStore: TokenStore): OkHttpClient =
        OkHttpClient.Builder()
            .connectTimeout(15, TimeUnit.SECONDS)
            // Локальная модель может отвечать долго — не обрываем соединение.
            .readTimeout(300, TimeUnit.SECONDS)
            .addInterceptor(AuthInterceptor(tokenStore))
            .build()

    fun createApi(client: OkHttpClient, baseUrl: String): ApiService =
        Retrofit.Builder()
            .baseUrl(baseUrl)
            .client(client)
            .addConverterFactory(json.asConverterFactory("application/json".toMediaType()))
            .build()
            .create(ApiService::class.java)
}

/** Добавляет Bearer-токен к каждому запросу (кроме анонимных эндпоинтов). */
class AuthInterceptor(private val tokenStore: TokenStore) : Interceptor {
    override fun intercept(chain: Interceptor.Chain): Response {
        val original = chain.request()
        val token = tokenStore.accessToken
        val request = if (token.isNullOrBlank()) {
            original
        } else {
            original.newBuilder()
                .header("Authorization", "Bearer $token")
                .build()
        }
        return chain.proceed(request)
    }
}
