package com.servicesphere.mobile.data.repository

import com.servicesphere.mobile.data.api.MobileApi
import com.servicesphere.mobile.data.model.CreateRequestDto
import com.servicesphere.mobile.data.model.MobileHomeResponseDto
import com.servicesphere.mobile.data.model.MobileRequestCardDto
import com.servicesphere.mobile.data.model.MobileRequestDetailDto
import com.servicesphere.mobile.data.model.UpdateStatusDto

class RequestRepository(
    private val api: MobileApi
) {
    suspend fun loadHome(): Result<MobileHomeResponseDto> {
        val response = api.getHome()
        return response.body()?.let(Result.Companion::success)
            ?: Result.failure(IllegalStateException("Не удалось загрузить home"))
    }

    suspend fun loadRequests(): Result<List<MobileRequestCardDto>> {
        val response = api.getRequests()
        return response.body()?.items?.let(Result.Companion::success)
            ?: Result.failure(IllegalStateException("Не удалось загрузить заявки"))
    }

    suspend fun loadRequest(id: String): Result<MobileRequestDetailDto> {
        val response = api.getRequest(id)
        return response.body()?.let(Result.Companion::success)
            ?: Result.failure(IllegalStateException("Не удалось загрузить заявку"))
    }

    suspend fun createRequest(title: String, description: String): Result<Unit> {
        val response = api.createRequest(CreateRequestDto(title = title, description = description))
        return if (response.isSuccessful) {
            Result.success(Unit)
        } else {
            Result.failure(IllegalStateException("Не удалось создать заявку. Проверьте, есть ли объект в системе"))
        }
    }

    suspend fun updateStatus(id: String, status: String): Result<Unit> {
        val response = api.changeStatus(id, UpdateStatusDto(status))
        return if (response.isSuccessful) {
            Result.success(Unit)
        } else {
            Result.failure(IllegalStateException("Не удалось сменить статус"))
        }
    }
}
