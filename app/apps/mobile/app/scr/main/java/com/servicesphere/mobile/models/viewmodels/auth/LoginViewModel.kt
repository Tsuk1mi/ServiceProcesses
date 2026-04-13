package com.servicesphere.mobile.viewmodels.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.servicesphere.mobile.repository.AuthRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class LoginViewModel(
    private val repository: AuthRepository
) : ViewModel() {

    private val _login = MutableStateFlow("")
    val login: StateFlow<String> = _login

    private val _password = MutableStateFlow("")
    val password: StateFlow<String> = _password

    private val _loading = MutableStateFlow(false)
    val loading: StateFlow<Boolean> = _loading

    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error

    fun updateLogin(value: String) {
        _login.value = value
    }

    fun updatePassword(value: String) {
        _password.value = value
    }

    fun login(onSuccess: () -> Unit) {

        viewModelScope.launch {

            _loading.value = true
            _error.value = null

            val result = repository.login(
                _login.value.trim(),
                _password.value.trim()
            )

            _loading.value = false

            result.onSuccess {
                onSuccess()
            }

            result.onFailure {
                _error.value = it.message
            }
        }
    }
}