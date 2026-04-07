package com.servicesphere.mobile.services

import android.content.Context
import android.content.SharedPreferences

class AuthService(context: Context) {

    private val prefs: SharedPreferences =
        context.getSharedPreferences("auth_storage", Context.MODE_PRIVATE)

    fun saveAccessToken(token: String) {
        prefs.edit().putString("access_token", token).apply()
    }

    fun saveRefreshToken(token: String?) {
        prefs.edit().putString("refresh_token", token).apply()
    }

    fun getAccessToken(): String? {
        return prefs.getString("access_token", null)
    }

    fun clear() {
        prefs.edit().clear().apply()
    }

}