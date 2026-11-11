package com.mentor.app

import android.app.Application
import android.content.Context
import com.mentor.core.network.ApiFactory
import com.mentor.core.network.ApiService
import com.mentor.core.network.ServerSettings
import com.mentor.core.network.SseChatClient
import com.mentor.core.network.TokenStore
import okhttp3.OkHttpClient

/**
 * Простой сервис-локатор (без DI-фреймворка на этапе каркаса):
 * собирает сетевой слой и хранилища.
 */
class AppContainer(context: Context) {
    val tokenStore = TokenStore(context)
    val serverSettings = ServerSettings(context)
    val okHttpClient: OkHttpClient = ApiFactory.createClient(tokenStore)
    val sseClient = SseChatClient(okHttpClient)

    var api: ApiService = ApiFactory.createApi(okHttpClient, serverSettings.baseUrl)
        private set

    /** Смена адреса бэкенда (применяется при следующем создании api). */
    fun updateBaseUrl(url: String) {
        serverSettings.baseUrl = url
        api = ApiFactory.createApi(okHttpClient, serverSettings.baseUrl)
    }

    fun currentBaseUrl(): String = serverSettings.baseUrl
}

class MentorApplication : Application() {
    lateinit var container: AppContainer
        private set

    override fun onCreate() {
        super.onCreate()
        container = AppContainer(this)
    }
}
