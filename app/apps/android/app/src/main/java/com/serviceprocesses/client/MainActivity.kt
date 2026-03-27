package com.serviceprocesses.client

import android.os.Bundle
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import java.net.HttpURLConnection
import java.net.URL

/**
 * Каркас клиента: проверка доступности backend по GET /health.
 * URL для эмулятора по умолчанию: 10.0.2.2:8080 (см. BuildConfig.API_BASE_URL).
 */
class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val status = TextView(this).apply {
            textSize = 15f
            setPadding(48, 48, 48, 24)
            text = getString(R.string.hint_tap_check)
        }
        val button = Button(this).apply {
            text = getString(R.string.action_check_api)
        }
        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            addView(status)
            addView(button)
        }
        setContentView(layout)

        button.setOnClickListener {
            status.text = getString(R.string.status_checking)
            Thread {
                try {
                    val base = BuildConfig.API_BASE_URL.trimEnd('/')
                    val conn = (URL("$base/health").openConnection() as HttpURLConnection).apply {
                        requestMethod = "GET"
                        connectTimeout = 8000
                        readTimeout = 8000
                    }
                    val code = conn.responseCode
                    val body = (if (code in 200..299) conn.inputStream else conn.errorStream)
                        .bufferedReader()
                        .use { it.readText() }
                    val healthy = code == 200 &&
                        body.contains("\"status\"", ignoreCase = true) &&
                        body.contains("ok", ignoreCase = true)
                    runOnUiThread {
                        status.text = if (healthy) {
                            getString(R.string.status_ok, body)
                        } else {
                            getString(R.string.status_http_error, code, body)
                        }
                    }
                } catch (e: Exception) {
                    runOnUiThread {
                        status.text = getString(R.string.status_error, e.message ?: e.toString())
                    }
                }
            }.start()
        }
    }
}
