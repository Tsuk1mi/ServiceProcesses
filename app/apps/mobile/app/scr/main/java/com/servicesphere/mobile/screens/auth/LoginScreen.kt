@Composable
fun LoginScreen(
    viewModel: LoginViewModel,
    onLoginSuccess: () -> Unit
) {

    val login by viewModel.login.collectAsState()
    val password by viewModel.password.collectAsState()
    val loading by viewModel.loading.collectAsState()
    val error by viewModel.error.collectAsState()

    Column(modifier = Modifier.padding(16.dp)) {

        OutlinedTextField(
            value = login,
            onValueChange = { viewModel.updateLogin(it) },
            label = { Text("Логин") }
        )

        OutlinedTextField(
            value = password,
            onValueChange = { viewModel.updatePassword(it) },
            label = { Text("Пароль") }
        )

        Button(
            onClick = { viewModel.login(onLoginSuccess) }
        ) {
            Text("Войти")
        }

        if (loading) {
            Text("Авторизация...")
        }

        error?.let {
            Text(it)
        }
    }
}

@Preview(showBackground = true)
@Composable
fun LoginScreenPreview() {

    LoginScreenContent(
        login = "",
        password = "",
        onLoginChange = {},
        onPasswordChange = {},
        onLoginClick = {}
    )

}