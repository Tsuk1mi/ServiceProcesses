package com.servicesphere.mobile.core

import com.servicesphere.mobile.BuildConfig

object Config {
    const val SESSION_STORAGE = "service_sphere_mobile_session"
    val baseUrl: String = BuildConfig.API_BASE_URL
}
