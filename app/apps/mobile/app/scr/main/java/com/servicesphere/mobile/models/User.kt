package com.servicesphere.mobile.models

data class LoginRequest(
    val login: String,
    val password: String
)

data class LoginResponse(
    val accessToken: String,
    val refreshToken: String?,
    val expiresIn: Long,
    val user: User
)

data class User(
    val id: Long,
    val name: String,
    val role: String
)