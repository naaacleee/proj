package com.mentor.core.network

import android.content.Context

/** Хранилище токенов текущей сессии. */
class TokenStore(context: Context) {
    private val prefs = context.getSharedPreferences("mentor_tokens", Context.MODE_PRIVATE)

    var accessToken: String?
        get() = prefs.getString(KEY_ACCESS, null)
        set(value) {
            prefs.edit().putString(KEY_ACCESS, value).apply()
        }

    var refreshToken: String?
        get() = prefs.getString(KEY_REFRESH, null)
        set(value) {
            prefs.edit().putString(KEY_REFRESH, value).apply()
        }

    var username: String?
        get() = prefs.getString(KEY_USERNAME, null)
        set(value) {
            prefs.edit().putString(KEY_USERNAME, value).apply()
        }

    fun hasSession(): Boolean = !refreshToken.isNullOrBlank()

    fun clear() {
        prefs.edit().clear().apply()
    }

    private companion object {
        const val KEY_ACCESS = "access_token"
        const val KEY_REFRESH = "refresh_token"
        const val KEY_USERNAME = "username"
    }
}
