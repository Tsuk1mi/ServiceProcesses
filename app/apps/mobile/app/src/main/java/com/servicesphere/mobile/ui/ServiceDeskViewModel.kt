package com.servicesphere.mobile.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.servicesphere.mobile.core.AuthService
import com.servicesphere.mobile.data.model.MobileCountersDto
import com.servicesphere.mobile.data.model.MobileRequestCardDto
import com.servicesphere.mobile.data.model.MobileRequestDetailDto
import com.servicesphere.mobile.data.repository.AuthRepository
import com.servicesphere.mobile.data.repository.RequestRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

enum class MobileScreen {
    LOGIN,
    HOME,
    REQUESTS,
    CREATE_REQUEST,
    REQUEST_DETAIL,
    PROFILE
}

data class SessionUiState(
    val loading: Boolean = true,
    val authenticated: Boolean = false,
    val userName: String = "admin",
    val userEmail: String = "admin@service.local",
    val userRole: String = "ADMIN",
    val error: String? = null
)

data class HomeUiState(
    val loading: Boolean = false,
    val counters: MobileCountersDto = MobileCountersDto(0, 0, 0),
    val recentRequests: List<MobileRequestCardDto> = emptyList(),
    val error: String? = null
)

data class RequestListUiState(
    val loading: Boolean = false,
    val items: List<MobileRequestCardDto> = emptyList(),
    val error: String? = null
)

data class RequestDetailUiState(
    val loading: Boolean = false,
    val item: MobileRequestDetailDto? = null,
    val error: String? = null
)

data class CreateRequestUiState(
    val saving: Boolean = false,
    val error: String? = null,
    val successMessage: String? = null
)

class ServiceDeskViewModel(
    private val authRepository: AuthRepository,
    private val requestRepository: RequestRepository,
    private val authService: AuthService
) : ViewModel() {
    private val _screen = MutableStateFlow(MobileScreen.LOGIN)
    val screen: StateFlow<MobileScreen> = _screen.asStateFlow()

    private val _session = MutableStateFlow(SessionUiState())
    val session: StateFlow<SessionUiState> = _session.asStateFlow()

    private val _home = MutableStateFlow(HomeUiState())
    val home: StateFlow<HomeUiState> = _home.asStateFlow()

    private val _requestList = MutableStateFlow(RequestListUiState())
    val requestList: StateFlow<RequestListUiState> = _requestList.asStateFlow()

    private val _requestDetail = MutableStateFlow(RequestDetailUiState())
    val requestDetail: StateFlow<RequestDetailUiState> = _requestDetail.asStateFlow()

    private val _createRequest = MutableStateFlow(CreateRequestUiState())
    val createRequest: StateFlow<CreateRequestUiState> = _createRequest.asStateFlow()

    private var currentRequestId: String? = null

    fun restoreSession() {
        viewModelScope.launch {
            if (!authService.hasSession()) {
                _session.value = SessionUiState(loading = false, authenticated = false)
                _screen.value = MobileScreen.LOGIN
                return@launch
            }

            _session.value = _session.value.copy(loading = true, error = null)
            authRepository.restoreSession()
                .onSuccess { user ->
                    if (!isAllowedMobileRole(user.role)) {
                        authService.clear()
                        _session.value = SessionUiState(
                            loading = false,
                            authenticated = false,
                            error = "Мобильное приложение доступно только клиентам и администраторам."
                        )
                        _screen.value = MobileScreen.LOGIN
                        return@onSuccess
                    }
                    _session.value = SessionUiState(
                        loading = false,
                        authenticated = true,
                        userName = user.name,
                        userEmail = user.email,
                        userRole = user.role
                    )
                    _screen.value = MobileScreen.HOME
                    refreshDashboard()
                }
                .onFailure {
                    _session.value = SessionUiState(
                        loading = false,
                        authenticated = false,
                        error = "Сессия истекла. Выполните вход заново."
                    )
                    _screen.value = MobileScreen.LOGIN
                }
        }
    }

    fun login(username: String, password: String) {
        viewModelScope.launch {
            _session.value = _session.value.copy(loading = true, error = null)
            authRepository.login(username.trim(), password.trim())
                .onSuccess { user ->
                    if (!isAllowedMobileRole(user.role)) {
                        authService.clear()
                        _session.value = SessionUiState(
                            loading = false,
                            authenticated = false,
                            error = "Мобильное приложение доступно только клиентам и администраторам."
                        )
                        _screen.value = MobileScreen.LOGIN
                        return@onSuccess
                    }
                    _session.value = SessionUiState(
                        loading = false,
                        authenticated = true,
                        userName = user.name,
                        userEmail = user.email,
                        userRole = user.role
                    )
                    _screen.value = MobileScreen.HOME
                    refreshDashboard()
                }
                .onFailure {
                    _session.value = _session.value.copy(
                        loading = false,
                        authenticated = false,
                        error = it.message ?: "Не удалось выполнить вход"
                    )
                }
        }
    }

    fun logout() {
        viewModelScope.launch {
            authRepository.logout()
            _screen.value = MobileScreen.LOGIN
            _session.value = SessionUiState(loading = false, authenticated = false)
            _home.value = HomeUiState()
            _requestList.value = RequestListUiState()
            _requestDetail.value = RequestDetailUiState()
            _createRequest.value = CreateRequestUiState()
            currentRequestId = null
        }
    }

    fun navigate(screen: MobileScreen) {
        _screen.value = screen
        when (screen) {
            MobileScreen.HOME -> refreshDashboard()
            MobileScreen.REQUESTS -> loadRequests()
            MobileScreen.CREATE_REQUEST -> _createRequest.value = CreateRequestUiState()
            MobileScreen.LOGIN, MobileScreen.REQUEST_DETAIL, MobileScreen.PROFILE -> Unit
        }
    }

    fun refreshDashboard() {
        loadHome()
        loadRequests()
    }

    private fun loadHome() {
        viewModelScope.launch {
            _home.value = _home.value.copy(loading = true, error = null)
            requestRepository.loadHome()
                .onSuccess { response ->
                    _home.value = HomeUiState(
                        loading = false,
                        counters = response.counters,
                        recentRequests = response.recentRequests
                    )
                }
                .onFailure {
                    _home.value = _home.value.copy(
                        loading = false,
                        error = it.message ?: "Не удалось загрузить главную"
                    )
                }
        }
    }

    fun loadRequests() {
        viewModelScope.launch {
            _requestList.value = _requestList.value.copy(loading = true, error = null)
            requestRepository.loadRequests()
                .onSuccess { items ->
                    _requestList.value = RequestListUiState(loading = false, items = items)
                }
                .onFailure {
                    _requestList.value = _requestList.value.copy(
                        loading = false,
                        error = it.message ?: "Не удалось загрузить заявки"
                    )
                }
        }
    }

    fun openRequest(id: String) {
        currentRequestId = id
        _screen.value = MobileScreen.REQUEST_DETAIL
        viewModelScope.launch {
            _requestDetail.value = RequestDetailUiState(loading = true, error = null)
            requestRepository.loadRequest(id)
                .onSuccess { item ->
                    _requestDetail.value = RequestDetailUiState(loading = false, item = item)
                }
                .onFailure {
                    _requestDetail.value = RequestDetailUiState(
                        loading = false,
                        error = it.message ?: "Не удалось загрузить заявку"
                    )
                }
        }
    }

    fun updateRequestStatus(status: String) {
        val requestId = currentRequestId ?: return
        viewModelScope.launch {
            _requestDetail.value = _requestDetail.value.copy(loading = true, error = null)
            requestRepository.updateStatus(requestId, status)
                .onSuccess {
                    openRequest(requestId)
                    loadRequests()
                    loadHome()
                }
                .onFailure {
                    _requestDetail.value = _requestDetail.value.copy(
                        loading = false,
                        error = it.message ?: "Не удалось обновить статус"
                    )
                }
        }
    }

    fun createRequest(title: String, description: String) {
        if (title.isBlank()) {
            _createRequest.value = _createRequest.value.copy(error = "Введите заголовок заявки")
            return
        }

        viewModelScope.launch {
            _createRequest.value = CreateRequestUiState(saving = true)
            requestRepository.createRequest(title.trim(), description.trim())
                .onSuccess {
                    _createRequest.value = CreateRequestUiState(
                        saving = false,
                        successMessage = "Заявка отправлена. Она появится после обновления read-модели."
                    )
                    loadHome()
                    loadRequests()
                    _screen.value = MobileScreen.REQUESTS
                }
                .onFailure {
                    _createRequest.value = CreateRequestUiState(
                        saving = false,
                        error = it.message ?: "Не удалось создать заявку"
                    )
                }
        }
    }

    class Factory(
        private val authRepository: AuthRepository,
        private val requestRepository: RequestRepository,
        private val authService: AuthService
    ) : ViewModelProvider.Factory {
        override fun <T : ViewModel> create(modelClass: Class<T>): T {
            if (modelClass.isAssignableFrom(ServiceDeskViewModel::class.java)) {
                @Suppress("UNCHECKED_CAST")
                return ServiceDeskViewModel(authRepository, requestRepository, authService) as T
            }
            throw IllegalArgumentException("Unknown ViewModel class: ${modelClass.name}")
        }
    }
}

private fun isAllowedMobileRole(role: String): Boolean {
    return role.equals("CLIENT", ignoreCase = true) || role.equals("ADMIN", ignoreCase = true)
}
