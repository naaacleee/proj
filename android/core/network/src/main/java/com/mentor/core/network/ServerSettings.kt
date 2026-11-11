package com.mentor.core.network

import android.content.Context

/** Настройки подключения к бэкенду. */
class ServerSettings(context: Context) {
    private val prefs = context.getSharedPreferences("mentor_settings", Context.MODE_PRIVATE)

    var baseUrl: String
        get() = prefs.getString(KEY_BASE_URL, null)?.takeIf { it.isNotBlank() }
            ?: BuildConfig.DEFAULT_BASE_URL
        set(value) {
            prefs.edit().putString(KEY_BASE_URL, normalize(value)).apply()
        }

    private fun normalize(url: String): String = url.trim().let {
        if (it.endsWith("/")) it else "$it/"
    }

    private companion object {
        const val KEY_BASE_URL = "base_url"
    }
}
