package com.mentor.feature.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.viewModelScope
import com.mentor.core.network.ApiService
import com.mentor.core.network.LoginRequest
import com.mentor.core.network.RegisterRequest
import com.mentor.core.network.TokenStore
import com.mentor.core.network.describeError
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

sealed interface AuthUiState {
    data object Idle : AuthUiState
    data object Loading : AuthUiState
    data class Error(val message: String) : AuthUiState
}

class AuthViewModel(
    private val api: ApiService,
    private val tokenStore: TokenStore,
) : ViewModel() {

    private val _state = MutableStateFlow<AuthUiState>(AuthUiState.Idle)
    val state: StateFlow<AuthUiState> = _state

    fun login(username: String, password: String, onSuccess: () -> Unit) =
        authenticate(username, password, register = false, onSuccess = onSuccess)

    fun register(username: String, password: String, onSuccess: () -> Unit) =
        authenticate(username, password, register = true, onSuccess = onSuccess)

    private fun authenticate(
        username: String,
        password: String,
        register: Boolean,
        onSuccess: () -> Unit,
    ) {
        val name = username.trim()
        if (name.length < 3) {
            _state.value = AuthUiState.Error("Имя пользователя — минимум 3 символа")
            return
        }
        if (password.length < 8) {
            _state.value = AuthUiState.Error("Пароль — минимум 8 символов")
            return
        }
        _state.value = AuthUiState.Loading
        viewModelScope.launch {
            try {
                val response = if (register) {
                    api.register(RegisterRequest(name, password))
                } else {
                    api.login(LoginRequest(name, password))
                }
                tokenStore.accessToken = response.access_token
                tokenStore.refreshToken = response.refresh_token
                tokenStore.username = response.user.username
                _state.value = AuthUiState.Idle
                onSuccess()
            } catch (t: Throwable) {
                _state.value = AuthUiState.Error(describeError(t))
            }
        }
    }
}

class AuthViewModelFactory(
    private val api: ApiService,
    private val tokenStore: TokenStore,
) : ViewModelProvider.Factory {
    @Suppress("UNCHECKED_CAST")
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(AuthViewModel::class.java)) {
            return AuthViewModel(api, tokenStore) as T
        }
        throw IllegalArgumentException("unknown ViewModel: ${modelClass.name}")
    }
}
