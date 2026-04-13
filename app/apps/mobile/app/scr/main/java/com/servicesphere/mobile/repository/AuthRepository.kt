package com.servicesphere.mobile.repository

import com.servicesphere.mobile.api.AuthApi
import com.servicesphere.mobile.models.LoginRequest
import com.servicesphere.mobile.services.AuthService

class AuthRepository(
    private val api: AuthApi,
    private val authService: AuthService
) {

    suspend fun login(
        login: String,
        password: String
    ): Result<Unit> {

        return try {

            val response = api.login(
                LoginRequest(login, password)
            )

            if (response.isSuccessful) {

                val body = response.body()

                if (body != null) {

                    authService.saveAccessToken(body.accessToken)
                    authService.saveRefreshToken(body.refreshToken)

                    Result.success(Unit)

                } else {
                    Result.failure(Exception("Empty response"))
                }

            } else {
                Result.failure(Exception("Authentication failed"))
            }

        } catch (e: Exception) {

            Result.failure(e)

        }
    }
}