package com.servicesphere.mobile.api

import com.servicesphere.mobile.Config
import com.servicesphere.mobile.services.AuthService
import okhttp3.Interceptor
import okhttp3.OkHttpClient
import okhttp3.Response
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory

class AuthInterceptor(
    private val authService: AuthService
) : Interceptor {

    override fun intercept(chain: Interceptor.Chain): Response {

        val request = chain.request()
        val token = authService.getAccessToken()

        val newRequest = if (token != null) {

            request.newBuilder()
                .addHeader("Authorization", "Bearer $token")
                .build()

        } else request

        return chain.proceed(newRequest)
    }
}

class ApiClient(authService: AuthService) {

    private val client = OkHttpClient.Builder()
        .addInterceptor(AuthInterceptor(authService))
        .build()

    private val retrofit = Retrofit.Builder()
        .baseUrl(Config.BASE_URL)
        .client(client)
        .addConverterFactory(GsonConverterFactory.create())
        .build()

    val authApi: AuthApi = retrofit.create(AuthApi::class.java)

}