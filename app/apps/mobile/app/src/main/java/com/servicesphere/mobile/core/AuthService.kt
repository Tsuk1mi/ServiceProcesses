package com.servicesphere.mobile.core

import android.content.Context
import android.content.SharedPreferences

class AuthService(context: Context) {
    private val preferences: SharedPreferences =
        context.getSharedPreferences(Config.SESSION_STORAGE, Context.MODE_PRIVATE)

    fun saveSession(
        accessToken: String,
        refreshToken: String?,
        userRole: String,
        userName: String,
        userEmail: String
    ) {
        preferences.edit()
            .putString(KEY_ACCESS_TOKEN, accessToken)
            .putString(KEY_REFRESH_TOKEN, refreshToken)
            .putString(KEY_USER_ROLE, userRole)
            .putString(KEY_USER_NAME, userName)
            .putString(KEY_USER_EMAIL, userEmail)
            .apply()
    }

    fun getAccessToken(): String? = preferences.getString(KEY_ACCESS_TOKEN, null)

    fun getRefreshToken(): String? = preferences.getString(KEY_REFRESH_TOKEN, null)

    fun getUserRole(): String = preferences.getString(KEY_USER_ROLE, "ADMIN") ?: "ADMIN"

    fun getUserName(): String = preferences.getString(KEY_USER_NAME, "admin") ?: "admin"

    fun getUserEmail(): String = preferences.getString(KEY_USER_EMAIL, "admin@service.local") ?: "admin@service.local"

    fun hasSession(): Boolean = !getAccessToken().isNullOrBlank()

    fun clear() {
        preferences.edit().clear().apply()
    }

    private companion object {
        const val KEY_ACCESS_TOKEN = "access_token"
        const val KEY_REFRESH_TOKEN = "refresh_token"
        const val KEY_USER_ROLE = "user_role"
        const val KEY_USER_NAME = "user_name"
        const val KEY_USER_EMAIL = "user_email"
    }
}
