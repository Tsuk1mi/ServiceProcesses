package com.servicesphere.mobile.data.repository

import com.servicesphere.mobile.core.AuthService
import com.servicesphere.mobile.data.api.MobileApi
import com.servicesphere.mobile.data.model.LoginRequestDto
import com.servicesphere.mobile.data.model.MobileUserDto
import com.servicesphere.mobile.data.model.RefreshRequestDto

class AuthRepository(
    private val api: MobileApi,
    private val authService: AuthService
) {
    suspend fun restoreSession(): Result<MobileUserDto> {
        if (!authService.hasSession()) {
            return Result.failure(IllegalStateException("Нет активной сессии"))
        }

        val response = api.me()
        if (response.isSuccessful) {
            val body = response.body()
            if (body != null) {
                authService.saveSession(
                    accessToken = authService.getAccessToken().orEmpty(),
                    refreshToken = authService.getRefreshToken(),
                    userRole = body.role,
                    userName = body.name,
                    userEmail = body.email
                )
                return Result.success(body)
            }
        }

        val refreshToken = authService.getRefreshToken()
        if (!refreshToken.isNullOrBlank()) {
            val refreshResponse = api.refresh(RefreshRequestDto(refreshToken))
            if (refreshResponse.isSuccessful) {
                val refreshBody = refreshResponse.body()
                if (refreshBody != null) {
                    authService.saveSession(
                        accessToken = refreshBody.accessToken,
                        refreshToken = refreshBody.refreshToken,
                        userRole = refreshBody.user.role,
                        userName = refreshBody.user.name,
                        userEmail = refreshBody.user.email
                    )
                    return Result.success(refreshBody.user)
                }
            }
        }

        authService.clear()
        return Result.failure(IllegalStateException("Сессия истекла"))
    }

    suspend fun login(username: String, password: String): Result<MobileUserDto> {
        val response = api.login(LoginRequestDto(username = username, password = password))
        if (!response.isSuccessful) {
            return Result.failure(IllegalStateException("Не удалось выполнить вход"))
        }

        val body = response.body() ?: return Result.failure(IllegalStateException("Пустой ответ сервера"))
        authService.saveSession(
            accessToken = body.accessToken,
            refreshToken = body.refreshToken,
            userRole = body.user.role,
            userName = body.user.name,
            userEmail = body.user.email
        )
        return Result.success(body.user)
    }

    suspend fun logout() {
        runCatching { api.logout() }
        authService.clear()
    }
}
