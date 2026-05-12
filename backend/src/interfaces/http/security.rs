use std::collections::{HashMap, VecDeque};
use std::env;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{HeaderValue, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use once_cell::sync::Lazy;
use tower_http::cors::{Any, CorsLayer};

use crate::domain::errors::DomainError;

use super::ErrorResponse;

static RATE_LIMITER: Lazy<SimpleRateLimiter> = Lazy::new(SimpleRateLimiter::default);

#[derive(Default)]
struct SimpleRateLimiter {
    windows: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl SimpleRateLimiter {
    fn allow(&self, bucket: &str, key: &str, limit: usize, window: Duration) -> bool {
        let now = Instant::now();
        let cutoff = now.checked_sub(window).unwrap_or(now);
        let mut guard = self.windows.lock().expect("rate limiter mutex poisoned");
        let entry = guard
            .entry(format!("{bucket}:{key}"))
            .or_default();
        while matches!(entry.front(), Some(ts) if *ts < cutoff) {
            entry.pop_front();
        }
        if entry.len() >= limit {
            return false;
        }
        entry.push_back(now);
        true
    }
}

pub async fn rate_limit_middleware(request: Request<Body>, next: Next) -> Response {
    let bucket = request.uri().path();
    let limit = if bucket == "/auth/login" {
        5
    } else if bucket.starts_with("/api/v1/bff/") {
        90
    } else if bucket.starts_with("/api/v1/query/") {
        120
    } else {
        240
    };
    let key = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .or_else(|| {
            request
                .headers()
                .get("authorization")
                .and_then(|value| value.to_str().ok())
        })
        .unwrap_or("global");

    if !RATE_LIMITER.allow(bucket, key, limit, Duration::from_secs(60)) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse {
                message: "rate limit exceeded".to_string(),
            }),
        )
            .into_response();
    }

    next.run(request).await
}

pub fn build_cors_layer() -> CorsLayer {
    let configured = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| {
        "http://localhost:5173,http://127.0.0.1:5173,http://localhost:4173".to_string()
    });
    if configured.trim() == "*" {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
            .allow_headers(Any);
    }

    let origins = configured
        .split(',')
        .filter_map(|value| HeaderValue::from_str(value.trim()).ok())
        .collect::<Vec<_>>();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any)
}

pub fn validate_required_text(field: &'static str, value: &str, max_len: usize) -> Result<(), DomainError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DomainError::EmptyField(field));
    }
    if trimmed.len() > max_len {
        return Err(DomainError::InvalidInput(field));
    }
    if trimmed.contains('<') || trimmed.contains('>') {
        return Err(DomainError::InvalidInput(field));
    }
    Ok(())
}

pub fn validate_string_enum(field: &'static str, value: &str, allowed: &[&str]) -> Result<(), DomainError> {
    if allowed.iter().any(|item| item.eq_ignore_ascii_case(value.trim())) {
        return Ok(());
    }
    Err(DomainError::InvalidInput(field))
}

pub fn validate_string_list(field: &'static str, values: &[String], max_len: usize) -> Result<(), DomainError> {
    if values.is_empty() {
        return Err(DomainError::EmptyField(field));
    }
    if values.iter().any(|value| value.trim().is_empty() || value.len() > max_len || value.contains('<') || value.contains('>')) {
        return Err(DomainError::InvalidInput(field));
    }
    Ok(())
}
