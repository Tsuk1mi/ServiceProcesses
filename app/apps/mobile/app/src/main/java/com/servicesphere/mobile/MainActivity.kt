package com.servicesphere.mobile

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import com.servicesphere.mobile.core.AuthService
import com.servicesphere.mobile.data.api.ApiClient
import com.servicesphere.mobile.data.repository.AuthRepository
import com.servicesphere.mobile.data.repository.RequestRepository
import com.servicesphere.mobile.ui.MobileScreen
import com.servicesphere.mobile.ui.ServiceDeskViewModel
import com.servicesphere.mobile.ui.screens.CreateRequestScreen
import com.servicesphere.mobile.ui.screens.HomeScreen
import com.servicesphere.mobile.ui.screens.LoginScreen
import com.servicesphere.mobile.ui.screens.ProfileScreen
import com.servicesphere.mobile.ui.screens.RequestDetailScreen
import com.servicesphere.mobile.ui.screens.RequestListScreen
import com.servicesphere.mobile.ui.theme.ServiceSphereTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        val authService = AuthService(applicationContext)
        val apiClient = ApiClient(authService)
        val authRepository = AuthRepository(apiClient.mobileApi, authService)
        val requestRepository = RequestRepository(apiClient.mobileApi)

        setContent {
            ServiceSphereTheme {
                val viewModel: ServiceDeskViewModel = viewModel(
                    factory = ServiceDeskViewModel.Factory(authRepository, requestRepository, authService)
                )
                ServiceDeskApp(viewModel)
            }
        }
    }
}

@Composable
private fun ServiceDeskApp(viewModel: ServiceDeskViewModel) {
    val session by viewModel.session.collectAsState()
    val screen by viewModel.screen.collectAsState()
    val home by viewModel.home.collectAsState()
    val requestList by viewModel.requestList.collectAsState()
    val requestDetail by viewModel.requestDetail.collectAsState()
    val createRequest by viewModel.createRequest.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.restoreSession()
    }

    if (!session.authenticated) {
        LoginScreen(
            state = session,
            onLogin = viewModel::login
        )
        return
    }

    Scaffold(
        containerColor = androidx.compose.material3.MaterialTheme.colorScheme.background,
        bottomBar = {
            NavigationBar(
                containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surface,
                contentColor = androidx.compose.material3.MaterialTheme.colorScheme.onSurface
            ) {
                MobileNavItem("Home", screen == MobileScreen.HOME) {
                    viewModel.navigate(MobileScreen.HOME)
                }
                MobileNavItem("Заявки", screen == MobileScreen.REQUESTS || screen == MobileScreen.REQUEST_DETAIL) {
                    viewModel.navigate(MobileScreen.REQUESTS)
                }
                MobileNavItem("Создать", screen == MobileScreen.CREATE_REQUEST) {
                    viewModel.navigate(MobileScreen.CREATE_REQUEST)
                }
                MobileNavItem("Профиль", screen == MobileScreen.PROFILE) {
                    viewModel.navigate(MobileScreen.PROFILE)
                }
            }
        }
    ) { padding ->
        Box(modifier = Modifier.padding(padding)) {
            when (screen) {
                MobileScreen.LOGIN -> LoginScreen(
                    state = session,
                    onLogin = viewModel::login
                )

                MobileScreen.HOME -> HomeScreen(
                    state = home,
                    userRole = session.userRole,
                    onRefresh = viewModel::refreshDashboard,
                    onOpenRequest = viewModel::openRequest,
                    onOpenRequests = { viewModel.navigate(MobileScreen.REQUESTS) },
                    onOpenCreate = { viewModel.navigate(MobileScreen.CREATE_REQUEST) }
                )

                MobileScreen.REQUESTS -> RequestListScreen(
                    state = requestList,
                    onRefresh = viewModel::loadRequests,
                    onOpenRequest = viewModel::openRequest
                )

                MobileScreen.REQUEST_DETAIL -> RequestDetailScreen(
                    state = requestDetail,
                    onRefresh = {
                        requestDetail.item?.ticketId?.let(viewModel::openRequest)
                    },
                    canChangeStatus = session.userRole.equals("ADMIN", ignoreCase = true),
                    onChangeStatus = viewModel::updateRequestStatus
                )

                MobileScreen.CREATE_REQUEST -> CreateRequestScreen(
                    state = createRequest,
                    onCreateRequest = viewModel::createRequest
                )

                MobileScreen.PROFILE -> ProfileScreen(
                    state = session,
                    onRefresh = viewModel::refreshDashboard,
                    onLogout = viewModel::logout
                )
            }
        }
    }
}

@Composable
private fun MobileNavItem(
    label: String,
    selected: Boolean,
    onClick: () -> Unit
) {
    TextButton(onClick = onClick) {
        Text(
            if (selected) "[$label]" else label,
            color = if (selected)
                androidx.compose.material3.MaterialTheme.colorScheme.primary
            else
                androidx.compose.material3.MaterialTheme.colorScheme.onSurface
        )
    }
}
