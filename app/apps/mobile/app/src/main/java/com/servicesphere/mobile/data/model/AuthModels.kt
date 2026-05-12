package com.servicesphere.mobile.data.model

data class LoginRequestDto(
    val username: String,
    val password: String
)

data class LoginResponseDto(
    val accessToken: String,
    val refreshToken: String?,
    val expiresIn: Long,
    val user: MobileUserDto
)

data class MobileUserDto(
    val id: String,
    val name: String,
    val email: String,
    val role: String,
    val status: String
)

data class RefreshRequestDto(
    val refresh_token: String
)
