package com.servicesphere.mobile.data.api

import com.servicesphere.mobile.core.AuthService
import com.servicesphere.mobile.core.Config
import okhttp3.Interceptor
import okhttp3.OkHttpClient
import okhttp3.Response
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory

private class AuthInterceptor(
    private val authService: AuthService
) : Interceptor {
    override fun intercept(chain: Interceptor.Chain): Response {
        val source = chain.request()
        val token = authService.getAccessToken()
        val request = if (token.isNullOrBlank()) {
            source
        } else {
            source.newBuilder()
                .addHeader("Authorization", "Bearer $token")
                .build()
        }
        return chain.proceed(request)
    }
}

class ApiClient(authService: AuthService) {
    private val httpClient = OkHttpClient.Builder()
        .addInterceptor(AuthInterceptor(authService))
        .addInterceptor(HttpLoggingInterceptor().apply {
            level = HttpLoggingInterceptor.Level.BASIC
        })
        .build()

    private val retrofit = Retrofit.Builder()
        .baseUrl(Config.baseUrl)
        .client(httpClient)
        .addConverterFactory(GsonConverterFactory.create())
        .build()

    val mobileApi: MobileApi = retrofit.create(MobileApi::class.java)
}
