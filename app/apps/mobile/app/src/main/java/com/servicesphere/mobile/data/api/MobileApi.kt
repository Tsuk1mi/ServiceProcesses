package com.servicesphere.mobile.data.api

import com.servicesphere.mobile.data.model.AcceptedResponseDto
import com.servicesphere.mobile.data.model.CreateRequestDto
import com.servicesphere.mobile.data.model.LoginRequestDto
import com.servicesphere.mobile.data.model.LoginResponseDto
import com.servicesphere.mobile.data.model.MobileHomeResponseDto
import com.servicesphere.mobile.data.model.MobileRequestDetailDto
import com.servicesphere.mobile.data.model.MobileRequestListResponseDto
import com.servicesphere.mobile.data.model.MobileUserDto
import com.servicesphere.mobile.data.model.RefreshRequestDto
import com.servicesphere.mobile.data.model.UpdateStatusDto
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path

interface MobileApi {
    @POST("/api/v1/bff/mobile/auth/login")
    suspend fun login(@Body request: LoginRequestDto): Response<LoginResponseDto>

    @POST("/api/v1/bff/mobile/auth/refresh")
    suspend fun refresh(@Body request: RefreshRequestDto): Response<LoginResponseDto>

    @POST("/api/v1/bff/mobile/auth/logout")
    suspend fun logout(): Response<Map<String, Boolean>>

    @GET("/api/v1/bff/mobile/auth/me")
    suspend fun me(): Response<MobileUserDto>

    @GET("/api/v1/bff/mobile/home")
    suspend fun getHome(): Response<MobileHomeResponseDto>

    @GET("/api/v1/bff/mobile/requests")
    suspend fun getRequests(): Response<MobileRequestListResponseDto>

    @GET("/api/v1/bff/mobile/requests/{id}")
    suspend fun getRequest(@Path("id") id: String): Response<MobileRequestDetailDto>

    @POST("/api/v1/bff/mobile/requests")
    suspend fun createRequest(@Body request: CreateRequestDto): Response<AcceptedResponseDto>

    @POST("/api/v1/bff/mobile/requests/{id}/status")
    suspend fun changeStatus(
        @Path("id") id: String,
        @Body body: UpdateStatusDto
    ): Response<AcceptedResponseDto>
}
