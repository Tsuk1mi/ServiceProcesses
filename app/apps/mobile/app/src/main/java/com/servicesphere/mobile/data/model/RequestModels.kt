package com.servicesphere.mobile.data.model

import com.google.gson.annotations.SerializedName

data class MobileHomeResponseDto(
    val counters: MobileCountersDto,
    @SerializedName("recent_requests")
    val recentRequests: List<MobileRequestCardDto>
)

data class MobileCountersDto(
    @SerializedName("open_requests")
    val openRequests: Int,
    @SerializedName("overdue_requests")
    val overdueRequests: Int,
    @SerializedName("active_work_orders")
    val activeWorkOrders: Int
)

data class MobileRequestListResponseDto(
    val items: List<MobileRequestCardDto>
)

data class MobileRequestCardDto(
    @SerializedName("ticket_id")
    val ticketId: String,
    val title: String,
    val description: String,
    val status: String
)

data class MobileRequestDetailDto(
    @SerializedName("ticket_id")
    val ticketId: String,
    val title: String,
    val status: String,
    val priority: String,
    val description: String,
    val overdue: Boolean,
    val assignee: String?
)

data class CreateRequestDto(
    val title: String,
    val description: String
)

data class UpdateStatusDto(
    val status: String
)

data class AcceptedResponseDto(
    val result: String
)
