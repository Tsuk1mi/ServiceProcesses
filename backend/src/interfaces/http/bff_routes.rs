use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{Asset, Escalation, WorkOrder};
use crate::ports::data_scope::DataScope;
use crate::ports::inbound::{CreateRequestCommand, ServiceRequestUseCase};
use crate::query::{ReadRequestDetail, ReadRequestListItem, RequestListFilter};

use super::query_routes::QueryRequestListQuery;
use super::{
    AppState, ErrorResponse, JwtAuth, LoginRequest, RefreshRequest, current_identity,
    domain_error_to_response, issue_tokens_from_credentials, issue_tokens_from_refresh, parse_status,
    revoke_current_session,
};

#[derive(Debug, Serialize)]
struct BffLoginResponse {
    accessToken: String,
    refreshToken: Option<String>,
    expiresIn: i64,
    user: BffUser,
}

#[derive(Debug, Serialize, Clone)]
struct BffUser {
    id: String,
    name: String,
    email: String,
    role: String,
    status: String,
    team: Option<String>,
    workload: Option<usize>,
}

#[derive(Debug, Serialize)]
struct WebDashboardResponse {
    newRequests: usize,
    inProgress: usize,
    slaBreached: usize,
    completedToday: usize,
    slaCompliance: f64,
    workloadByTechnician: Vec<WebWorkloadItem>,
    activity: Vec<WebActivityItem>,
}

#[derive(Debug, Serialize)]
struct WebRequestListResponse {
    items: Vec<WebRequestCard>,
    overdueCount: usize,
    total: usize,
}

#[derive(Debug, Serialize)]
struct WebWorkloadItem {
    name: String,
    closed: usize,
}

#[derive(Debug, Serialize)]
struct WebActivityItem {
    id: String,
    text: String,
    at: String,
}

#[derive(Debug, Serialize)]
struct MobileHomeResponse {
    counters: MobileCounters,
    recent_requests: Vec<MobileRequestCard>,
}

#[derive(Debug, Serialize)]
struct MobileCounters {
    open_requests: usize,
    overdue_requests: usize,
    active_work_orders: usize,
}

#[derive(Debug, Serialize)]
struct MobileRequestListResponse {
    items: Vec<MobileRequestCard>,
}

#[derive(Debug, Serialize)]
struct MobileRequestDetailResponse {
    ticket_id: String,
    title: String,
    status: String,
    priority: String,
    description: String,
    overdue: bool,
    assignee: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MobileCreateRequestBody {
    title: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct MobileStatusBody {
    status: String,
}

#[derive(Debug, Serialize)]
struct WebRequestCard {
    id: String,
    title: String,
    r#type: String,
    category: String,
    objectId: String,
    objectName: String,
    requester: String,
    status: String,
    priority: String,
    slaStatus: String,
    assignee: Option<String>,
    createdAt: String,
    dueAt: String,
}

#[derive(Debug, Serialize)]
struct WebWorkOrderDto {
    id: String,
    requestId: String,
    objectName: String,
    assignee: String,
    status: String,
    plannedStart: String,
    actualHours: Option<f64>,
    tasks: Vec<WebTaskDto>,
}

#[derive(Debug, Serialize)]
struct WebTaskDto {
    id: String,
    title: String,
    done: bool,
}

#[derive(Debug, Serialize)]
struct WebObjectDto {
    id: String,
    name: String,
    r#type: String,
    serialNumber: String,
    address: String,
    status: String,
    manufacturer: String,
    model: String,
    installedAt: String,
}

#[derive(Debug, Serialize)]
struct WebEscalationDto {
    id: String,
    requestId: String,
    level: String,
    reason: String,
    target: String,
    elapsedMinutes: usize,
}

#[derive(Debug, Serialize)]
struct WebNotificationDto {
    id: String,
    r#type: String,
    title: String,
    body: String,
    href: String,
    read: bool,
    createdAt: String,
}

#[derive(Debug, Serialize)]
struct WebSlaPolicyDto {
    id: String,
    name: String,
    objectType: String,
    reactionMinutes: HashMap<String, usize>,
    resolutionMinutes: HashMap<String, usize>,
    schedule: String,
}

#[derive(Debug, Serialize)]
struct DesktopTicketDto {
    id: String,
    objectName: String,
    status: String,
    priority: String,
    assignedTo: String,
}

#[derive(Debug, Serialize)]
struct MobileRequestCard {
    ticket_id: String,
    title: String,
    description: String,
    status: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/bff/web/auth/login", post(web_login))
        .route("/api/v1/bff/web/auth/refresh", post(web_refresh))
        .route("/api/v1/bff/web/auth/logout", post(web_logout))
        .route("/api/v1/bff/web/auth/me", get(web_me))
        .route("/api/v1/bff/mobile/auth/login", post(mobile_login))
        .route("/api/v1/bff/mobile/auth/refresh", post(mobile_refresh))
        .route("/api/v1/bff/mobile/auth/logout", post(mobile_logout))
        .route("/api/v1/bff/mobile/auth/me", get(mobile_me))
        .route("/api/v1/bff/desktop/auth/login", post(desktop_login))
        .route("/api/v1/bff/desktop/auth/refresh", post(desktop_refresh))
        .route("/api/v1/bff/desktop/auth/logout", post(desktop_logout))
        .route("/api/v1/bff/desktop/auth/me", get(desktop_me))
        .route("/api/v1/bff/web/dashboard", get(web_dashboard))
        .route("/api/v1/bff/web/requests", get(web_requests))
        .route("/api/v1/bff/web/requests/:id", get(web_request_detail))
        .route("/api/v1/bff/web/work-orders", get(web_work_orders))
        .route("/api/v1/bff/web/work-orders/:id", get(web_work_order_detail))
        .route("/api/v1/bff/web/objects", get(web_objects))
        .route("/api/v1/bff/web/objects/:id", get(web_object_detail))
        .route("/api/v1/bff/web/escalations", get(web_escalations))
        .route("/api/v1/bff/web/sla/policies", get(web_sla_policies))
        .route("/api/v1/bff/web/sla/breaches", get(web_sla_breaches))
        .route("/api/v1/bff/web/analytics/overview", get(web_analytics_overview))
        .route("/api/v1/bff/web/users", get(web_users))
        .route("/api/v1/bff/web/notifications", get(web_notifications))
        .route("/api/v1/bff/mobile/home", get(mobile_home))
        .route("/api/v1/bff/mobile/requests", get(mobile_requests).post(mobile_create_request))
        .route("/api/v1/bff/mobile/requests/:id", get(mobile_request_detail))
        .route("/api/v1/bff/mobile/requests/:id/status", post(mobile_change_request_status))
        .route("/api/v1/bff/desktop/tickets", get(desktop_tickets))
        .route("/api/v1/bff/desktop/health", get(desktop_health))
}

async fn web_login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> impl IntoResponse {
    bff_login(state, body, "web").await
}

async fn mobile_login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> impl IntoResponse {
    bff_login(state, body, "mobile").await
}

async fn desktop_login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> impl IntoResponse {
    bff_login(state, body, "desktop").await
}

async fn bff_login(state: AppState, body: LoginRequest, client: &str) -> axum::response::Response {
    let username = normalize_login(&body.username);
    match issue_tokens_from_credentials(&state, &username, &body.password, client).await {
        Ok(tokens) => {
            state.metrics.record_command(&format!("bff_{client}_login"));
            (
                axum::http::StatusCode::OK,
                Json(BffLoginResponse {
                    accessToken: tokens.access_token,
                    refreshToken: Some(tokens.refresh_token),
                    expiresIn: tokens.expires_in,
                    user: map_bff_user(&tokens.identity.username, &tokens.identity.auth.roles, None),
                }),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn web_refresh(State(state): State<AppState>, Json(body): Json<RefreshRequest>) -> impl IntoResponse {
    bff_refresh(state, body, "web").await
}

async fn mobile_refresh(State(state): State<AppState>, Json(body): Json<RefreshRequest>) -> impl IntoResponse {
    bff_refresh(state, body, "mobile").await
}

async fn desktop_refresh(State(state): State<AppState>, Json(body): Json<RefreshRequest>) -> impl IntoResponse {
    bff_refresh(state, body, "desktop").await
}

async fn bff_refresh(state: AppState, body: RefreshRequest, client: &str) -> axum::response::Response {
    match issue_tokens_from_refresh(&state, &body.refresh_token).await {
        Ok(tokens) => {
            state.metrics.record_command(&format!("bff_{client}_refresh"));
            (
                axum::http::StatusCode::OK,
                Json(BffLoginResponse {
                    accessToken: tokens.access_token,
                    refreshToken: Some(tokens.refresh_token),
                    expiresIn: tokens.expires_in,
                    user: map_bff_user(&tokens.identity.username, &tokens.identity.auth.roles, None),
                }),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn web_logout(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    bff_logout(state, auth, "web").await
}

async fn mobile_logout(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    bff_logout(state, auth, "mobile").await
}

async fn desktop_logout(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    bff_logout(state, auth, "desktop").await
}

async fn bff_logout(state: AppState, auth: JwtAuth, client: &str) -> axum::response::Response {
    match revoke_current_session(&state, &auth.0).await {
        Ok(()) => {
            state.metrics.record_command(&format!("bff_{client}_logout"));
            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({ "ok": true })),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn web_me(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    bff_me(state, auth, "web").await
}

async fn mobile_me(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    bff_me(state, auth, "mobile").await
}

async fn desktop_me(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    bff_me(state, auth, "desktop").await
}

async fn bff_me(state: AppState, auth: JwtAuth, client: &str) -> axum::response::Response {
    match current_identity(&state, &auth.0).await {
        Ok(identity) => {
            state.metrics.record_query(&format!("bff_{client}_me"));
            (
                axum::http::StatusCode::OK,
                Json(map_bff_user(
                    &identity.username,
                    &identity.auth.roles,
                    None,
                )),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn web_dashboard(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    let scope = auth.data_scope();
    let summary = match state.query_models.dashboard_summary(super::now_epoch(), scope).await {
        Ok(value) => value,
        Err(e) => return domain_error_to_response(e),
    };
    let sla = match state.query_models.sla_compliance_summary(super::now_epoch(), scope).await {
        Ok(value) => value,
        Err(e) => return domain_error_to_response(e),
    };
    let urgent: Vec<WebRequestCard> = match state
        .query_models
        .list_requests(
            super::now_epoch(),
            scope,
            RequestListFilter {
                limit: 5,
                offset: 0,
                status: None,
                priority: Some("Critical".to_string()),
                overdue_only: false,
            },
        )
        .await
    {
        Ok(items) => items.into_iter().map(to_web_card).collect(),
        Err(e) => return domain_error_to_response(e),
    };
    let workload = match state.query_models.technician_workload(scope).await {
        Ok(value) => value,
        Err(e) => return domain_error_to_response(e),
    };
    state.metrics.record_query("bff_web_dashboard");
    (
        axum::http::StatusCode::OK,
        Json(WebDashboardResponse {
            newRequests: summary.total_requests.saturating_sub(summary.in_progress_requests + summary.closed_requests + summary.resolved_requests),
            inProgress: summary.in_progress_requests,
            slaBreached: summary.overdue_requests,
            completedToday: summary.closed_requests + summary.resolved_requests,
            slaCompliance: sla.compliance_percent,
            workloadByTechnician: workload
                .into_iter()
                .map(|item| WebWorkloadItem {
                    name: item.full_name,
                    closed: item.completed,
                })
                .collect(),
            activity: urgent
                .into_iter()
                .take(5)
                .enumerate()
                .map(|(index, item)| WebActivityItem {
                    id: format!("act-{}-{}", index, item.id),
                    text: format!("{}: {} / {}", item.id, item.objectName, item.status),
                    at: item.createdAt.clone(),
                })
                .collect(),
        }),
    )
        .into_response()
}

async fn web_requests(
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
            let overdue_count = items.iter().filter(|item| item.overdue).count();
            let total = items.len();
            state.metrics.record_query("bff_web_requests");
            (
                axum::http::StatusCode::OK,
                Json(WebRequestListResponse {
                    items: items.into_iter().map(to_web_card).collect(),
                    overdueCount: overdue_count,
                    total,
                }),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_request_detail(
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
            state.metrics.record_query("bff_web_request_detail");
            (
                axum::http::StatusCode::OK,
                Json(to_web_request_from_detail(item)),
            )
                .into_response()
        }
        Ok(None) => domain_error_to_response(crate::domain::errors::DomainError::NotFound("bff_request")),
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_work_orders(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    let scope = auth.data_scope();
    match build_work_orders(&state, scope).await {
        Ok(items) => {
            state.metrics.record_query("bff_web_work_orders");
            (axum::http::StatusCode::OK, Json(items)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_work_order_detail(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match build_work_orders(&state, auth.data_scope()).await {
        Ok(items) => {
            if let Some(item) = items.into_iter().find(|item| item.id == id) {
                state.metrics.record_query("bff_web_work_order_detail");
                (axum::http::StatusCode::OK, Json(item)).into_response()
            } else {
                domain_error_to_response(crate::domain::errors::DomainError::NotFound("work_order"))
            }
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_objects(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    match state.assets.list(auth.data_scope()).await {
        Ok(items) => {
            state.metrics.record_query("bff_web_objects");
            (
                axum::http::StatusCode::OK,
                Json(items.into_iter().map(to_web_object).collect::<Vec<_>>()),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_object_detail(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.assets.get_by_id(&id, auth.data_scope()).await {
        Ok(Some(item)) => {
            state.metrics.record_query("bff_web_object_detail");
            (axum::http::StatusCode::OK, Json(to_web_object(item))).into_response()
        }
        Ok(None) => domain_error_to_response(crate::domain::errors::DomainError::NotFound("asset")),
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_escalations(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    match state.escalation_service.list_all(auth.data_scope()).await {
        Ok(items) => {
            state.metrics.record_query("bff_web_escalations");
            (
                axum::http::StatusCode::OK,
                Json(items.into_iter().map(to_web_escalation).collect::<Vec<_>>()),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_sla_policies(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    let policies = build_sla_policies(auth.primary_role_for_audit());
    state.metrics.record_query("bff_web_sla_policies");
    (axum::http::StatusCode::OK, Json(policies)).into_response()
}

async fn web_sla_breaches(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    match state
        .query_models
        .list_requests(
            super::now_epoch(),
            auth.data_scope(),
            RequestListFilter {
                overdue_only: true,
                limit: 100,
                ..Default::default()
            },
        )
        .await
    {
        Ok(items) => {
            state.metrics.record_query("bff_web_sla_breaches");
            (
                axum::http::StatusCode::OK,
                Json(items.into_iter().map(to_web_card).collect::<Vec<_>>()),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_analytics_overview(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    web_dashboard(State(state), auth).await
}

async fn web_users(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.query_models.technician_workload(scope).await {
        Ok(items) => {
            let mut users = items
                .into_iter()
                .map(|item| BffUser {
                    id: item.technician_id.clone(),
                    name: item.full_name,
                    email: format!("{}@service.local", item.technician_id),
                    role: "TECHNICIAN".to_string(),
                    status: "ACTIVE".to_string(),
                    team: Some("Field".to_string()),
                    workload: Some(item.total),
                })
                .collect::<Vec<_>>();
            users.insert(0, map_bff_user(&auth.sub.to_string(), &auth.roles, None));
            state.metrics.record_query("bff_web_users");
            (axum::http::StatusCode::OK, Json(users)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn web_notifications(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    let scope = auth.data_scope();
    let overdue = state
        .query_models
        .list_requests(
            super::now_epoch(),
            scope,
            RequestListFilter {
                overdue_only: true,
                limit: 5,
                ..Default::default()
            },
        )
        .await;
    let escalations = state.escalation_service.list_all(scope).await;
    match (overdue, escalations) {
        (Ok(requests), Ok(escalations)) => {
            let mut notifications = requests
                .into_iter()
                .enumerate()
                .map(|(index, item)| WebNotificationDto {
                    id: format!("notif-req-{}", index),
                    r#type: "SLA".to_string(),
                    title: format!("SLA под угрозой: {}", item.request_id),
                    body: format!("{} / {}", item.asset_title, item.description),
                    href: format!("/requests/{}", item.request_id),
                    read: false,
                    createdAt: format_epoch(item.created_at_epoch_sec),
                })
                .collect::<Vec<_>>();
            notifications.extend(escalations.into_iter().take(5).enumerate().map(|(index, item)| WebNotificationDto {
                id: format!("notif-esc-{}", index),
                r#type: "ESCALATION".to_string(),
                title: format!("Эскалация {}", item.id),
                body: item.reason,
                href: format!("/requests/{}", item.request_id),
                read: false,
                createdAt: "now".to_string(),
            }));
            state.metrics.record_query("bff_web_notifications");
            (axum::http::StatusCode::OK, Json(notifications)).into_response()
        }
        (Err(e), _) | (_, Err(e)) => domain_error_to_response(e),
    }
}

async fn mobile_home(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    let scope = auth.data_scope();
    let summary = match state.query_models.dashboard_summary(super::now_epoch(), scope).await {
        Ok(value) => value,
        Err(e) => return domain_error_to_response(e),
    };
    let recent_requests = match state
        .query_models
        .list_requests(super::now_epoch(), scope, RequestListFilter { limit: 5, ..Default::default() })
        .await
    {
        Ok(items) => items.into_iter().map(to_mobile_card).collect(),
        Err(e) => return domain_error_to_response(e),
    };
    state.metrics.record_query("bff_mobile_home");
    (
        axum::http::StatusCode::OK,
        Json(MobileHomeResponse {
            counters: MobileCounters {
                open_requests: summary.open_requests,
                overdue_requests: summary.overdue_requests,
                active_work_orders: summary.active_work_orders,
            },
            recent_requests,
        }),
    )
        .into_response()
}

async fn mobile_requests(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<QueryRequestListQuery>,
) -> impl IntoResponse {
    let filter = RequestListFilter {
        limit: query.limit.unwrap_or(50).min(200),
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
            state.metrics.record_query("bff_mobile_requests");
            (
                axum::http::StatusCode::OK,
                Json(MobileRequestListResponse {
                    items: items.into_iter().map(to_mobile_card).collect(),
                }),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn mobile_request_detail(
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
            state.metrics.record_query("bff_mobile_request_detail");
            (
                axum::http::StatusCode::OK,
                Json(MobileRequestDetailResponse {
                    ticket_id: item.request_id,
                    title: item.asset_title,
                    status: item.status,
                    priority: item.priority,
                    description: item.description,
                    overdue: item.overdue,
                    assignee: item.assignee,
                }),
            )
                .into_response()
        }
        Ok(None) => domain_error_to_response(crate::domain::errors::DomainError::NotFound("bff_mobile_request")),
        Err(e) => domain_error_to_response(e),
    }
}

async fn mobile_create_request(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<MobileCreateRequestBody>,
) -> impl IntoResponse {
    let asset_id = match state.assets.list(auth.data_scope()).await {
        Ok(items) => items.first().map(|item| item.id.clone()).unwrap_or_else(|| "asset-1".to_string()),
        Err(e) => return domain_error_to_response(e),
    };
    let description = if body.description.trim().is_empty() {
        body.title
    } else {
        format!("{}: {}", body.title, body.description)
    };
    let command = CreateRequestCommand {
        request_id: format!("req-{}", Uuid::new_v4().simple()),
        asset_id,
        description,
    };
    match state
        .service
        .create_request(&auth.0, command, auth.data_scope())
        .await
    {
        Ok(()) => {
            state.metrics.record_command("bff_mobile_create_request");
            (
                axum::http::StatusCode::ACCEPTED,
                Json(serde_json::json!({ "result": "accepted" })),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn mobile_change_request_status(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
    Json(body): Json<MobileStatusBody>,
) -> impl IntoResponse {
    let Some(status) = parse_status(&body.status) else {
        return domain_error_to_response(crate::domain::errors::DomainError::InvalidInput("status"));
    };
    match state
        .service
        .update_status(&auth.0, &id, status, auth.data_scope())
        .await
    {
        Ok(()) => {
            state.metrics.record_command("bff_mobile_change_request_status");
            (
                axum::http::StatusCode::ACCEPTED,
                Json(serde_json::json!({ "result": "accepted" })),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn desktop_tickets(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    match state
        .query_models
        .list_requests(super::now_epoch(), auth.data_scope(), RequestListFilter { limit: 100, ..Default::default() })
        .await
    {
        Ok(items) => {
            state.metrics.record_query("bff_desktop_tickets");
            (
                axum::http::StatusCode::OK,
                Json(items.into_iter().map(to_desktop_ticket).collect::<Vec<_>>()),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn desktop_health() -> impl IntoResponse {
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok"
        })),
    )
}

fn to_web_card(item: ReadRequestListItem) -> WebRequestCard {
    WebRequestCard {
        id: item.request_id,
        title: item.description.clone(),
        r#type: if item.priority == "Critical" { "EMERGENCY" } else { "PLANNED" }.to_string(),
        category: "SERVICE".to_string(),
        objectId: item.asset_id,
        objectName: item.asset_title,
        requester: "System".to_string(),
        status: map_request_status(&item.status),
        priority: map_priority(&item.priority),
        slaStatus: if item.overdue { "BREACHED".to_string() } else { "OK".to_string() },
        assignee: item.assignee,
        createdAt: format_epoch(item.created_at_epoch_sec),
        dueAt: format_epoch(item.sla_deadline_epoch_sec),
    }
}

fn to_web_request_from_detail(item: ReadRequestDetail) -> WebRequestCard {
    to_web_card(ReadRequestListItem {
        request_id: item.request_id,
        asset_id: item.asset_id,
        asset_title: item.asset_title,
        asset_location: item.asset_location,
        description: item.description,
        priority: item.priority,
        status: item.status,
        assignee: item.assignee,
        open_escalation_count: item.open_escalation_count,
        work_order_count: usize::from(item.work_order_status.is_some()),
        overdue: item.overdue,
        sla_deadline_epoch_sec: item.sla_deadline_epoch_sec,
        created_at_epoch_sec: item.created_at_epoch_sec,
        owner_user_id: item.owner_user_id,
    })
}

fn to_mobile_card(item: ReadRequestListItem) -> MobileRequestCard {
    MobileRequestCard {
        ticket_id: item.request_id,
        title: item.asset_title,
        description: item.description,
        status: map_request_status(&item.status),
    }
}

fn to_desktop_ticket(item: ReadRequestListItem) -> DesktopTicketDto {
    DesktopTicketDto {
        id: item.request_id,
        objectName: item.asset_title,
        status: map_request_status(&item.status),
        priority: map_priority(&item.priority),
        assignedTo: item.assignee.unwrap_or_else(|| "Не назначен".to_string()),
    }
}

async fn build_work_orders(state: &AppState, scope: DataScope) -> Result<Vec<WebWorkOrderDto>, crate::domain::errors::DomainError> {
    let requests = state
        .requests
        .list(scope)
        .await?
        .into_iter()
        .map(|item| (item.id.clone(), item))
        .collect::<HashMap<_, _>>();
    let assets = state
        .assets
        .list(scope)
        .await?
        .into_iter()
        .map(|item| (item.id.clone(), item))
        .collect::<HashMap<_, _>>();
    let items = state.work_orders.list(scope).await?;
    Ok(items
        .into_iter()
        .map(|item| to_web_work_order(item, &requests, &assets))
        .collect())
}

fn to_web_work_order(
    item: WorkOrder,
    requests: &HashMap<String, crate::domain::entities::ServiceRequest>,
    assets: &HashMap<String, Asset>,
) -> WebWorkOrderDto {
    let object_name = requests
        .get(&item.request_id)
        .and_then(|request| assets.get(&request.asset_id))
        .map(|asset| asset.title.clone())
        .unwrap_or_else(|| "Объект".to_string());
    WebWorkOrderDto {
        id: item.id.clone(),
        requestId: item.request_id,
        objectName: object_name,
        assignee: item.assignee.clone().unwrap_or_else(|| "Не назначен".to_string()),
        status: map_work_order_status(&format!("{:?}", item.status)),
        plannedStart: "2026-05-05T09:00:00Z".to_string(),
        actualHours: if format!("{:?}", item.status) == "Completed" { Some(4.0) } else { None },
        tasks: vec![
            WebTaskDto {
                id: format!("{}-task-1", item.id),
                title: "Осмотр".to_string(),
                done: true,
            },
            WebTaskDto {
                id: format!("{}-task-2", item.id),
                title: "Проверка".to_string(),
                done: format!("{:?}", item.status) == "Completed",
            },
        ],
    }
}

fn to_web_object(item: Asset) -> WebObjectDto {
    WebObjectDto {
        id: item.id.clone(),
        name: item.title.clone(),
        r#type: item.kind.clone(),
        serialNumber: format!("SN-{}", item.id),
        address: item.location.clone(),
        status: match format!("{:?}", item.state).as_str() {
            "Active" => "OPERATIONAL".to_string(),
            "Maintenance" => "MAINTENANCE".to_string(),
            _ => "FAILED".to_string(),
        },
        manufacturer: "ServiceProcesses".to_string(),
        model: item.kind,
        installedAt: "2025-01-01".to_string(),
    }
}

fn to_web_escalation(item: Escalation) -> WebEscalationDto {
    WebEscalationDto {
        id: item.id,
        requestId: item.request_id,
        level: "L1".to_string(),
        reason: if item.reason.to_ascii_lowercase().contains("sla") {
            "SLA_BREACH".to_string()
        } else {
            "MANUAL".to_string()
        },
        target: "supervisor".to_string(),
        elapsedMinutes: 15,
    }
}

fn build_sla_policies(prefix: &str) -> Vec<WebSlaPolicyDto> {
    ["building", "hvac", "electrical"]
        .iter()
        .enumerate()
        .map(|(index, object_type)| WebSlaPolicyDto {
            id: format!("sla-{}", index + 1),
            name: format!("{} policy {}", prefix, index + 1),
            objectType: (*object_type).to_string(),
            reactionMinutes: HashMap::from([
                ("CRITICAL".to_string(), 15),
                ("HIGH".to_string(), 30),
                ("MEDIUM".to_string(), 60),
                ("LOW".to_string(), 120),
            ]),
            resolutionMinutes: HashMap::from([
                ("CRITICAL".to_string(), 120),
                ("HIGH".to_string(), 240),
                ("MEDIUM".to_string(), 480),
                ("LOW".to_string(), 1440),
            ]),
            schedule: "TWENTY_FOUR_SEVEN".to_string(),
        })
        .collect()
}

fn map_bff_user(username: &str, roles: &[String], workload: Option<usize>) -> BffUser {
    let role = if roles.iter().any(|item| item == "admin") {
        "ADMIN"
    } else if roles.iter().any(|item| item == "dispatcher") {
        "DISPATCHER"
    } else if roles.iter().any(|item| item == "technician") {
        "TECHNICIAN"
    } else if roles.iter().any(|item| item == "supervisor") {
        "MANAGER"
    } else {
        "CLIENT"
    };
    BffUser {
        id: username.to_string(),
        name: username.to_string(),
        email: if username.contains('@') {
            username.to_string()
        } else {
            format!("{username}@service.local")
        },
        role: role.to_string(),
        status: "ACTIVE".to_string(),
        team: Some("Operations".to_string()),
        workload,
    }
}

fn normalize_login(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed
        .split('@')
        .next()
        .unwrap_or(trimmed)
        .to_string()
}

fn map_priority(priority: &str) -> String {
    priority.to_ascii_uppercase()
}

fn map_request_status(status: &str) -> String {
    match status {
        "New" => "NEW".to_string(),
        "Planned" => "PAUSED".to_string(),
        "InProgress" => "IN_PROGRESS".to_string(),
        "Resolved" | "Closed" => "COMPLETED".to_string(),
        "Escalated" => "ESCALATED".to_string(),
        other => other.to_ascii_uppercase(),
    }
}

fn map_work_order_status(status: &str) -> String {
    match status {
        "Created" => "NEW".to_string(),
        "Assigned" => "ASSIGNED".to_string(),
        "InProgress" => "IN_PROGRESS".to_string(),
        "Completed" => "COMPLETED".to_string(),
        "Cancelled" => "CANCELLED".to_string(),
        other => other.to_ascii_uppercase(),
    }
}

fn format_epoch(epoch: u64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(epoch as i64, 0)
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}
