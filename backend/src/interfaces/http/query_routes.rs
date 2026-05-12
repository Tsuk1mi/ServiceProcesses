use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::query::RequestListFilter;

use super::{domain_error_to_response, AppState, JwtAuth};

#[derive(Debug, Deserialize, Default)]
pub struct QueryRequestListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub overdue_only: Option<bool>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/query/requests", get(list_requests_query))
        .route("/api/v1/query/requests/:id", get(get_request_detail_query))
        .route("/api/v1/query/dashboard/summary", get(get_dashboard_summary_query))
        .route("/api/v1/query/dashboard/sla-compliance", get(get_sla_compliance_query))
        .route(
            "/api/v1/query/dashboard/sla-compliance-by-priority",
            get(get_sla_compliance_by_priority_query),
        )
        .route("/api/v1/query/dashboard/technicians/workload", get(get_workload_query))
        .route("/api/v1/query/heatmap", get(get_heatmap_query))
}

async fn list_requests_query(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<QueryRequestListQuery>,
) -> impl IntoResponse {
    let filter = RequestListFilter {
        limit: query.limit.unwrap_or(100).min(500),
        offset: query.offset.unwrap_or(0),
        status: query.status,
        priority: query.priority,
        overdue_only: query.overdue_only.unwrap_or(false),
    };
    match state
        .query_models
        .list_requests(super::now_epoch(), auth.data_scope(), filter)
        .await
    {
        Ok(items) => {
            state.metrics.record_query("list_requests");
            (axum::http::StatusCode::OK, Json(items)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn get_request_detail_query(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state
        .query_models
        .get_request_detail(super::now_epoch(), &id, auth.data_scope())
        .await
    {
        Ok(Some(item)) => {
            state.metrics.record_query("get_request_detail");
            (axum::http::StatusCode::OK, Json(item)).into_response()
        }
        Ok(None) => domain_error_to_response(crate::domain::errors::DomainError::NotFound("read_request_detail")),
        Err(e) => domain_error_to_response(e),
    }
}

async fn get_dashboard_summary_query(
    State(state): State<AppState>,
    auth: JwtAuth,
) -> impl IntoResponse {
    match state
        .query_models
        .dashboard_summary(super::now_epoch(), auth.data_scope())
        .await
    {
        Ok(item) => {
            state.metrics.record_query("dashboard_summary");
            (axum::http::StatusCode::OK, Json(item)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn get_sla_compliance_query(
    State(state): State<AppState>,
    auth: JwtAuth,
) -> impl IntoResponse {
    match state
        .query_models
        .sla_compliance_summary(super::now_epoch(), auth.data_scope())
        .await
    {
        Ok(item) => {
            state.metrics.record_query("sla_compliance");
            (axum::http::StatusCode::OK, Json(item)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn get_sla_compliance_by_priority_query(
    State(state): State<AppState>,
    auth: JwtAuth,
) -> impl IntoResponse {
    match state
        .query_models
        .sla_compliance_by_priority(super::now_epoch(), auth.data_scope())
        .await
    {
        Ok(item) => {
            state.metrics.record_query("sla_by_priority");
            (axum::http::StatusCode::OK, Json(item)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn get_workload_query(
    State(state): State<AppState>,
    auth: JwtAuth,
) -> impl IntoResponse {
    match state.query_models.technician_workload(auth.data_scope()).await {
        Ok(item) => {
            state.metrics.record_query("technician_workload");
            (axum::http::StatusCode::OK, Json(item)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn get_heatmap_query(State(state): State<AppState>, _auth: JwtAuth) -> impl IntoResponse {
    state.metrics.record_query("heatmap");
    (axum::http::StatusCode::OK, Json(state.metrics.snapshot())).into_response()
}
