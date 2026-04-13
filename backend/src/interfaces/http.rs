use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::header::{self, AUTHORIZATION};
use axum::http::request::Parts;
use axum::http::{Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use http_body_util::BodyExt;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{IntoParams, Modify, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use crate::auth::{sign_token, verify_token, AuthUser, UserStore};

use crate::application::audit_service::AuditAppService;
use crate::application::escalation_service::EscalationAppService;
use crate::application::analytics_snapshot_service::AnalyticsSnapshotAppService;
use crate::application::reporting_service::ReportingAppService;
use crate::application::service_request_service::ServiceRequestAppService;
use crate::application::sla_service::SlaAppService;
use crate::application::technician_service::TechnicianAppService;
use crate::application::work_order_service::WorkOrderAppService;
use crate::domain::analytics::{
    DashboardSummary, SlaComplianceByPriorityItem, SlaComplianceSummary, TechnicianWorkloadSummary,
};
use crate::domain::entities::{Asset, AuditRecord, Escalation, ServiceRequest, Technician, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::RequestStatus;
use crate::infrastructure::jobs::JobClient;
use crate::infrastructure::redis_cache::RedisCache;
use crate::ports::inbound::{CreateRequestCommand, ServiceRequestUseCase};
use crate::ports::outbound::{
    AssetRepository, AuditRepository, EscalationRepository, ServiceRequestRepository, TechnicianRepository,
    WorkOrderRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub assets: Arc<dyn AssetRepository>,
    pub requests: Arc<dyn ServiceRequestRepository>,
    pub service: ServiceRequestAppService,
    pub work_orders: Arc<dyn WorkOrderRepository>,
    pub work_order_service: WorkOrderAppService,
    pub escalations: Arc<dyn EscalationRepository>,
    pub escalation_service: EscalationAppService,
    pub technicians: Arc<dyn TechnicianRepository>,
    pub technician_service: TechnicianAppService,
    pub audit: Arc<dyn AuditRepository>,
    pub audit_service: AuditAppService,
    pub sla_service: SlaAppService,
    pub reporting_service: ReportingAppService,
    pub analytics_snapshot_service: AnalyticsSnapshotAppService,
    pub jwt_secret: Arc<String>,
    pub users: Arc<dyn UserStore>,
    /// RabbitMQ + Redis; если `None`, эндпоинты задач отвечают 503.
    pub jobs: Option<Arc<JobClient>>,
    /// Кэш GET `/api/v1/*` в Redis (при наличии `REDIS_URL` вместе с RabbitMQ).
    pub redis_cache: Option<Arc<RedisCache>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MutationResult {
    pub result: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedEscalationsResult {
    pub created: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAssetRequest {
    pub kind: String,
    pub title: String,
    pub location: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateServiceRequestRequest {
    pub asset_id: String,
    pub description: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateWorkOrderRequest {
    pub request_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignWorkOrderRequest {
    pub assignee: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEscalationRequest {
    pub request_id: String,
    pub reason: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTechnicianRequest {
    pub full_name: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddRoleRequest {
    pub subject_id: Uuid,
    pub role: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EnqueueJobRequest {
    /// `echo` или `simulate_slow`
    pub kind: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EnqueueJobResponse {
    pub job_id: Uuid,
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct JobStatusResponse {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Default, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize, Default, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RequestListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

fn api_cache_key(request: &Request<Body>) -> String {
    let pq = request.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("");
    let auth = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let mut hasher = Sha256::new();
    hasher.update(pq.as_bytes());
    hasher.update(b"|");
    hasher.update(auth.as_bytes());
    format!("cache:api:v1:{:x}", hasher.finalize())
}

fn should_cache_get_path(path: &str) -> bool {
    path.starts_with("/api/v1/") && !path.starts_with("/api/v1/jobs")
}

async fn redis_http_cache_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let method = request.method().clone();
    tracing::info!(
        method = %method,
        path = %request.uri().path(),
        "incoming http request"
    );

    let Some(ref cache) = state.redis_cache else {
        return next.run(request).await;
    };

    if method != Method::GET {
        cache.invalidate_api_cache().await;
        return next.run(request).await;
    }

    let key_for_store = if should_cache_get_path(request.uri().path()) {
        Some(api_cache_key(&request))
    } else {
        None
    };

    if let Some(ref k) = key_for_store {
        if let Some(bytes) = cache.get(k).await {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .header("X-Cache", "HIT")
                .body(Body::from(bytes))
                .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "cache").into_response());
        }
    }

    let res = next.run(request).await;

    let Some(key) = key_for_store else {
        return res;
    };
    if !res.status().is_success() {
        return res;
    }

    let ct_json = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_ascii_lowercase().contains("json"))
        .unwrap_or(false);
    if !ct_json {
        return res;
    }

    let (parts, body) = res.into_parts();
    match body.collect().await {
        Ok(col) => {
            let bytes = col.to_bytes();
            cache.set(&key, bytes.as_ref(), 60).await;
            Response::from_parts(parts, Body::from(bytes))
        }
        Err(_) => Response::from_parts(parts, Body::empty()),
    }
}

pub fn router(state: AppState) -> Router {
    let cache_mw_state = state.clone();
    let public = Router::new()
        .route("/health", get(health))
        .route("/auth/login", post(login))
        .with_state(state.clone());

    let api = Router::new()
        .route("/api/v1/assets", post(create_asset).get(list_assets))
        .route("/api/v1/assets/:id", get(get_asset))
        .route("/api/v1/requests", post(create_request).get(list_requests))
        .route("/api/v1/requests/overdue", get(list_overdue_requests))
        .route("/api/v1/requests/:id", get(get_request))
        .route("/api/v1/requests/:id/status", put(update_request_status))
        .route(
            "/api/v1/work-orders",
            post(create_work_order).get(list_work_orders),
        )
        .route("/api/v1/work-orders/:id/assign", put(assign_work_order))
        .route("/api/v1/work-orders/:id/start", put(start_work_order))
        .route("/api/v1/work-orders/:id/complete", put(complete_work_order))
        .route("/api/v1/requests/:id/work-orders", get(list_work_orders_by_request))
        .route(
            "/api/v1/escalations",
            post(create_escalation).get(list_escalations),
        )
        .route("/api/v1/sla/escalate-overdue", post(escalate_overdue_requests))
        .route("/api/v1/escalations/:id/resolve", put(resolve_escalation))
        .route("/api/v1/requests/:id/escalations", get(list_escalations_by_request))
        .route("/api/v1/technicians", post(create_technician).get(list_technicians))
        .route("/api/v1/requests/:id/audit", get(list_request_audit))
        .route("/api/v1/dashboard/summary", get(get_dashboard_summary))
        .route("/api/v1/dashboard/sla-compliance", get(get_sla_compliance_summary))
        .route(
            "/api/v1/dashboard/sla-compliance-by-priority",
            get(get_sla_compliance_by_priority_summary),
        )
        .route(
            "/api/v1/dashboard/technicians/workload",
            get(get_technician_workload_summary),
        )
        .route("/api/v1/admin/roles", post(admin_add_role))
        .route("/api/v1/jobs", post(enqueue_job))
        .route("/api/v1/jobs/:id", get(get_job_status))
        .with_state(state.clone())
        .layer(middleware::from_fn_with_state(
            cache_mw_state,
            redis_http_cache_middleware,
        ));

    Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .merge(public)
        .merge(api)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = 200, description = "Сервис доступен", body = HealthResponse))
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
    })
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "JWT", body = LoginResponse),
        (status = 401, body = ErrorResponse)
    )
)]
async fn login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> impl IntoResponse {
    let Some(user) = state.users.verify(&body.username, &body.password).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                message: "invalid credentials".to_string(),
            }),
        )
            .into_response();
    };
    let ttl = 24;
    match sign_token(state.jwt_secret.as_str(), &user, ttl) {
        Ok(token) => (
            StatusCode::OK,
            Json(LoginResponse {
                access_token: token,
                token_type: "Bearer".to_string(),
                expires_in: ttl * 3600,
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                message: "token issue failed".to_string(),
            }),
        )
            .into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/roles",
    tag = "admin",
    security(("bearer" = [])),
    request_body = AddRoleRequest,
    responses(
        (status = 200, description = "Роль добавлена (или уже была)"),
        (status = 401, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse)
    )
)]
async fn admin_add_role(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<AddRoleRequest>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin"]) {
        return r;
    }
    match state.users.add_role_for_subject(body.subject_id, &body.role).await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/jobs",
    tag = "jobs",
    security(("bearer" = [])),
    request_body = EnqueueJobRequest,
    responses(
        (status = 202, description = "Задача в очереди RabbitMQ, статус в Redis", body = EnqueueJobResponse),
        (status = 400, body = ErrorResponse),
        (status = 401, body = ErrorResponse),
        (status = 503, body = ErrorResponse)
    )
)]
async fn enqueue_job(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<EnqueueJobRequest>,
) -> impl IntoResponse {
    let k = body.kind.as_str();
    if k != "echo" && k != "simulate_slow" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                message: "разрешены только kind: echo, simulate_slow".to_string(),
            }),
        )
            .into_response();
    }
    let Some(ref jobs) = state.jobs else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                message: "очередь не настроена: задайте REDIS_URL и RABBITMQ_URL".to_string(),
            }),
        )
            .into_response();
    };
    let owner = auth.sub.to_string();
    match jobs
        .enqueue(body.kind, body.payload, owner)
        .await
    {
        Ok(job_id) => (
            StatusCode::ACCEPTED,
            Json(EnqueueJobResponse {
                job_id,
                status: "queued".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "enqueue job failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    message: "не удалось поставить задачу в очередь".to_string(),
                }),
            )
                .into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/jobs/{id}",
    tag = "jobs",
    security(("bearer" = [])),
    params(("id" = Uuid, Path, description = "UUID задачи")),
    responses(
        (status = 200, body = JobStatusResponse),
        (status = 401, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 503, body = ErrorResponse)
    )
)]
async fn get_job_status(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let Some(ref jobs) = state.jobs else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                message: "очередь не настроена: задайте REDIS_URL и RABBITMQ_URL".to_string(),
            }),
        )
            .into_response();
    };
    let rec = match jobs.get_status(&id).await {
        Ok(Some(r)) => r,
        Ok(None) => return domain_error_to_response(DomainError::NotFound("job")),
        Err(e) => {
            tracing::error!(error = %e, "redis get job status");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    message: "не удалось прочитать статус задачи".to_string(),
                }),
            )
                .into_response();
        }
    };
    if !auth.is_admin() && rec.owner_user_id != auth.sub.to_string() {
        return domain_error_to_response(DomainError::NotFound("job"));
    }
    let body = JobStatusResponse {
        status: rec.status,
        result: rec.result,
        error: rec.error,
    };
    (StatusCode::OK, Json(body)).into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/assets",
    tag = "assets",
    request_body = CreateAssetRequest,
    params(
        ("x-role" = Option<String>, Header, description = "dispatcher, supervisor, technician или viewer"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора для аудита")
    ),
    responses(
        (status = 201, description = "Создан", body = Asset),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse)
    )
)]
async fn create_asset(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateAssetRequest>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    let id = format!("asset-{}", Uuid::new_v4().simple());
    let owner = auth.sub.to_string();
    match Asset::new(id, body.kind, body.title, body.location, owner.clone()) {
        Ok(asset) => match state.assets.save(asset.clone(), auth.data_scope()).await {
            Ok(()) => {
                let _ = state
                    .audit_service
                    .record(
                        None,
                        "asset",
                        "create",
                        auth.primary_role_for_audit(),
                        Some(auth.sub.to_string()),
                        format!("asset_id={}", asset.id),
                        owner,
                    )
                    .await;
                (StatusCode::CREATED, Json(asset)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        },
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/assets",
    tag = "assets",
    responses(
        (status = 200, description = "Список активов", body = [Asset]),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_assets(State(state): State<AppState>, auth: JwtAuth) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.assets.list(scope).await {
        Ok(assets) => (StatusCode::OK, Json(assets)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/assets/{id}",
    tag = "assets",
    params(("id" = String, Path, description = "Идентификатор актива")),
    responses(
        (status = 200, body = Asset),
        (status = 404, body = ErrorResponse)
    )
)]
async fn get_asset(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.assets.get_by_id(&id, scope).await {
        Ok(Some(asset)) => (StatusCode::OK, Json(asset)).into_response(),
        Ok(None) => domain_error_to_response(DomainError::NotFound("asset")),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/requests",
    tag = "requests",
    request_body = CreateServiceRequestRequest,
    params(
        ("x-role" = Option<String>, Header, description = "dispatcher или supervisor"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора")
    ),
    responses(
        (status = 201, description = "Создана", body = MutationResult),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse)
    )
)]
async fn create_request(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateServiceRequestRequest>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "dispatcher", "supervisor", "user"]) {
        return r;
    }
    let scope = auth.data_scope();
    let request_id = format!("req-{}", Uuid::new_v4().simple());
    let command = CreateRequestCommand {
        request_id: request_id.clone(),
        asset_id: body.asset_id.clone(),
        description: body.description,
    };

    match state
        .service
        .create_request(&auth.0, command.clone(), scope.clone())
        .await
    {
        Ok(()) => {
            let audit_owner = state
                .assets
                .get_by_id(&body.asset_id, scope.clone())
                .await
                .ok()
                .flatten()
                .map(|a| a.owner_user_id)
                .unwrap_or_else(|| auth.sub.to_string());
            let _ = state
                .audit_service
                .record(
                    Some(command.request_id.clone()),
                    "service_request",
                    "create",
                    auth.primary_role_for_audit(),
                    Some(auth.sub.to_string()),
                    format!("asset_id={}", command.asset_id),
                    audit_owner,
                )
                .await;
            (
                StatusCode::CREATED,
                Json(MutationResult {
                    result: "created".into(),
                }),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/requests",
    tag = "requests",
    params(RequestListQuery),
    responses(
        (status = 200, body = [ServiceRequest]),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_requests(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<RequestListQuery>,
) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.requests.list(scope).await {
        Ok(requests) => {
            let filtered = requests
                .into_iter()
                .filter(|r| query.status.as_ref().map(|s| format!("{:?}", r.status).eq_ignore_ascii_case(s)).unwrap_or(true))
                .filter(|r| query.priority.as_ref().map(|p| format!("{:?}", r.priority).eq_ignore_ascii_case(p)).unwrap_or(true))
                .collect::<Vec<_>>();
            let sliced = apply_pagination(filtered, query.limit, query.offset);
            (StatusCode::OK, Json(sliced)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}",
    tag = "requests",
    params(("id" = String, Path, description = "Идентификатор заявки")),
    responses(
        (status = 200, body = ServiceRequest),
        (status = 404, body = ErrorResponse)
    )
)]
async fn get_request(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.requests.get_by_id(&id, scope).await {
        Ok(Some(request)) => (StatusCode::OK, Json(request)).into_response(),
        Ok(None) => domain_error_to_response(DomainError::NotFound("service_request")),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/requests/overdue",
    tag = "requests",
    params(ListQuery),
    responses(
        (status = 200, body = [ServiceRequest]),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_overdue_requests(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.sla_service.list_overdue_requests(now_epoch(), scope).await {
        Ok(overdue) => {
            (StatusCode::OK, Json(apply_pagination(overdue, query.limit, query.offset))).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/requests/{id}/status",
    tag = "requests",
    request_body = UpdateStatusRequest,
    params(
        ("id" = String, Path, description = "Идентификатор заявки"),
        ("x-role" = Option<String>, Header, description = "dispatcher или supervisor"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора")
    ),
    responses(
        (status = 200, body = MutationResult),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse)
    )
)]
async fn update_request_status(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
    Json(body): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    let scope = auth.data_scope();
    let next = match parse_status(&body.status) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    message: "unsupported status".to_string(),
                }),
            )
                .into_response()
        }
    };

    let audit_owner = match state.requests.get_by_id(&id, scope.clone()).await {
        Ok(Some(r)) => r.owner_user_id,
        _ => auth.sub.to_string(),
    };

    match state.service.update_status(&auth.0, &id, next, scope).await {
        Ok(()) => {
            let _ = state
                .audit_service
                .record(
                    Some(id.clone()),
                    "service_request",
                    "update_status",
                    auth.primary_role_for_audit(),
                    Some(auth.sub.to_string()),
                    format!("status={:?}", next),
                    audit_owner,
                )
                .await;
            (
                StatusCode::OK,
                Json(MutationResult {
                    result: "updated".into(),
                }),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/work-orders",
    tag = "work-orders",
    request_body = CreateWorkOrderRequest,
    params(
        ("x-role" = Option<String>, Header, description = "dispatcher или supervisor"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора")
    ),
    responses(
        (status = 201, body = WorkOrder),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse)
    )
)]
async fn create_work_order(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateWorkOrderRequest>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    let scope = auth.data_scope();
    let id = format!("wo-{}", Uuid::new_v4().simple());
    match state
        .work_order_service
        .create_work_order(&auth.0, id, body.request_id.clone(), scope.clone())
        .await
    {
        Ok(order) => {
            let audit_owner = order.owner_user_id.clone();
            let _ = state
                .audit_service
                .record(
                    Some(order.request_id.clone()),
                    "work_order",
                    "create",
                    auth.primary_role_for_audit(),
                    Some(auth.sub.to_string()),
                    format!("work_order_id={}", order.id),
                    audit_owner,
                )
                .await;
            (StatusCode::CREATED, Json(order)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/work-orders",
    tag = "work-orders",
    params(
        ListQuery,
        ("x-role" = Option<String>, Header, description = "Любая роль с доступом к чтению"),
        ("x-actor-id" = Option<String>, Header, description = "Опционально")
    ),
    responses(
        (status = 200, body = [WorkOrder]),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_work_orders(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(
        &auth,
        &[
            "admin",
            "supervisor",
            "dispatcher",
            "viewer",
            "technician",
            "user",
        ],
    ) {
        return r;
    }
    let scope = auth.data_scope();
    match state.work_orders.list(scope).await {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/work-orders",
    tag = "work-orders",
    params(
        ("id" = String, Path, description = "Идентификатор заявки"),
        ListQuery
    ),
    responses(
        (status = 200, body = [WorkOrder]),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_work_orders_by_request(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.work_order_service.list_by_request(&id, scope).await {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/work-orders/{id}/assign",
    tag = "work-orders",
    request_body = AssignWorkOrderRequest,
    params(
        ("id" = String, Path, description = "Идентификатор наряда"),
        ("x-role" = Option<String>, Header, description = "dispatcher или supervisor"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора")
    ),
    responses(
        (status = 200, body = WorkOrder),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse)
    )
)]
async fn assign_work_order(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
    Json(body): Json<AssignWorkOrderRequest>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    let scope = auth.data_scope();
    match state
        .work_order_service
        .assign(&auth.0, &id, body.assignee, scope.clone())
        .await
    {
        Ok(order) => {
            let audit_owner = order.owner_user_id.clone();
            let _ = state
                .audit_service
                .record(
                    Some(order.request_id.clone()),
                    "work_order",
                    "assign",
                    auth.primary_role_for_audit(),
                    Some(auth.sub.to_string()),
                    format!(
                        "work_order_id={},assignee={}",
                        order.id,
                        order.assignee.clone().unwrap_or_default()
                    ),
                    audit_owner,
                )
                .await;
            (StatusCode::OK, Json(order)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/work-orders/{id}/start",
    tag = "work-orders",
    params(
        ("id" = String, Path, description = "Идентификатор наряда"),
        ("x-role" = Option<String>, Header, description = "technician, dispatcher или supervisor"),
        ("x-actor-id" = Option<String>, Header, description = "Для technician — обязателен и должен совпадать с assignee")
    ),
    responses(
        (status = 200, body = WorkOrder),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse)
    )
)]
async fn start_work_order(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "technician", "dispatcher", "supervisor"]) {
        return r;
    }
    let scope = auth.data_scope();
    let is_technician_only = auth.has_any_role(&["technician"]) && !auth.is_admin() && !auth.has_any_role(&["dispatcher", "supervisor"]);

    if is_technician_only {
        let actor_id = auth.sub.to_string();
        match state
            .work_order_service
            .start_by_actor(&auth.0, &id, &actor_id, scope.clone())
            .await
        {
            Ok(order) => {
                let audit_owner = order.owner_user_id.clone();
                let _ = state
                    .audit_service
                    .record(
                        Some(order.request_id.clone()),
                        "work_order",
                        "start",
                        auth.primary_role_for_audit(),
                        Some(actor_id),
                        format!("work_order_id={}", order.id),
                        audit_owner,
                    )
                    .await;
                (StatusCode::OK, Json(order)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        }
    } else {
        match state.work_order_service.start(&auth.0, &id, scope.clone()).await {
            Ok(order) => {
                let audit_owner = order.owner_user_id.clone();
                let _ = state
                    .audit_service
                    .record(
                        Some(order.request_id.clone()),
                        "work_order",
                        "start",
                        auth.primary_role_for_audit(),
                        Some(auth.sub.to_string()),
                        format!("work_order_id={}", order.id),
                        audit_owner,
                    )
                    .await;
                (StatusCode::OK, Json(order)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/work-orders/{id}/complete",
    tag = "work-orders",
    params(
        ("id" = String, Path, description = "Идентификатор наряда"),
        ("x-role" = Option<String>, Header, description = "technician, dispatcher или supervisor"),
        ("x-actor-id" = Option<String>, Header, description = "Для technician — обязателен")
    ),
    responses(
        (status = 200, body = WorkOrder),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse)
    )
)]
async fn complete_work_order(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "technician", "dispatcher", "supervisor"]) {
        return r;
    }
    let scope = auth.data_scope();
    let is_technician_only = auth.has_any_role(&["technician"]) && !auth.is_admin() && !auth.has_any_role(&["dispatcher", "supervisor"]);

    if is_technician_only {
        let actor_id = auth.sub.to_string();
        match state
            .work_order_service
            .complete_by_actor(&auth.0, &id, &actor_id, scope.clone())
            .await
        {
            Ok(order) => {
                let audit_owner = order.owner_user_id.clone();
                let _ = state
                    .audit_service
                    .record(
                        Some(order.request_id.clone()),
                        "work_order",
                        "complete",
                        auth.primary_role_for_audit(),
                        Some(actor_id),
                        format!("work_order_id={}", order.id),
                        audit_owner,
                    )
                    .await;
                (StatusCode::OK, Json(order)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        }
    } else {
        match state.work_order_service.complete(&auth.0, &id, scope.clone()).await {
            Ok(order) => {
                let audit_owner = order.owner_user_id.clone();
                let _ = state
                    .audit_service
                    .record(
                        Some(order.request_id.clone()),
                        "work_order",
                        "complete",
                        auth.primary_role_for_audit(),
                        Some(auth.sub.to_string()),
                        format!("work_order_id={}", order.id),
                        audit_owner,
                    )
                    .await;
                (StatusCode::OK, Json(order)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        }
    }
}

/// Обёртка для извлечения JWT из запроса (обход правил сирот для `FromRequestParts`).
#[derive(Clone, Debug)]
pub struct JwtAuth(pub AuthUser);

impl std::ops::Deref for JwtAuth {
    type Target = AuthUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl FromRequestParts<AppState> for JwtAuth {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let raw = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        message: "missing Authorization header".to_string(),
                    }),
                )
            })?;

        let token = raw
            .strip_prefix("Bearer ")
            .or_else(|| raw.strip_prefix("bearer "))
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        message: "expected Bearer token".to_string(),
                    }),
                )
            })?;

        let user = verify_token(state.jwt_secret.as_str(), token.trim()).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    message: "invalid or expired token".to_string(),
                }),
            )
        })?;
        Ok(JwtAuth(user))
    }
}

fn require_roles(auth: &JwtAuth, allowed: &[&str]) -> Result<(), axum::response::Response> {
    if auth.0.has_any_role(allowed) {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            message: "operation is not allowed for current role".to_string(),
        }),
    )
        .into_response())
}

#[utoipa::path(
    post,
    path = "/api/v1/escalations",
    tag = "escalations",
    request_body = CreateEscalationRequest,
    params(
        ("x-role" = Option<String>, Header, description = "dispatcher или supervisor"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора")
    ),
    responses(
        (status = 201, body = Escalation),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse)
    )
)]
async fn create_escalation(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateEscalationRequest>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    let scope = auth.data_scope();
    let id = format!("esc-{}", Uuid::new_v4().simple());
    match state
        .escalation_service
        .create_escalation(&auth.0, id, body.request_id, body.reason, scope.clone())
        .await
    {
        Ok(escalation) => {
            let audit_owner = escalation.owner_user_id.clone();
            let _ = state
                .audit_service
                .record(
                    Some(escalation.request_id.clone()),
                    "escalation",
                    "create",
                    auth.primary_role_for_audit(),
                    Some(auth.sub.to_string()),
                    format!("escalation_id={}", escalation.id),
                    audit_owner,
                )
                .await;
            (StatusCode::CREATED, Json(escalation)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/escalations",
    tag = "escalations",
    params(
        ListQuery,
        ("x-role" = Option<String>, Header, description = "Любая роль с доступом к чтению"),
        ("x-actor-id" = Option<String>, Header, description = "Опционально")
    ),
    responses(
        (status = 200, body = [Escalation]),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_escalations(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(
        &auth,
        &[
            "admin",
            "supervisor",
            "dispatcher",
            "viewer",
            "technician",
            "user",
        ],
    ) {
        return r;
    }
    let scope = auth.data_scope();
    match state.escalation_service.list_all(scope).await {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/escalations",
    tag = "escalations",
    params(
        ("id" = String, Path, description = "Идентификатор заявки"),
        ListQuery
    ),
    responses(
        (status = 200, body = [Escalation]),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_escalations_by_request(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.escalation_service.list_by_request(&id, scope).await {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/escalations/{id}/resolve",
    tag = "escalations",
    params(
        ("id" = String, Path, description = "Идентификатор эскалации"),
        ("x-role" = Option<String>, Header, description = "supervisor или dispatcher"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора")
    ),
    responses(
        (status = 200, body = Escalation),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse)
    )
)]
async fn resolve_escalation(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "supervisor", "dispatcher"]) {
        return r;
    }
    let scope = auth.data_scope();
    match state.escalation_service.resolve(&auth.0, &id, scope.clone()).await {
        Ok(item) => {
            let audit_owner = item.owner_user_id.clone();
            let _ = state
                .audit_service
                .record(
                    Some(item.request_id.clone()),
                    "escalation",
                    "resolve",
                    auth.primary_role_for_audit(),
                    Some(auth.sub.to_string()),
                    format!("escalation_id={}", item.id),
                    audit_owner,
                )
                .await;
            (StatusCode::OK, Json(item)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/sla/escalate-overdue",
    tag = "sla",
    params(
        ("x-role" = Option<String>, Header, description = "supervisor или dispatcher"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора")
    ),
    responses(
        (status = 200, body = CreatedEscalationsResult),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
async fn escalate_overdue_requests(
    State(state): State<AppState>,
    auth: JwtAuth,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "supervisor", "dispatcher"]) {
        return r;
    }
    let created = match state
        .sla_service
        .auto_escalate_overdue(&auth.0, now_epoch(), "Automatic SLA overdue escalation")
        .await
    {
        Ok(v) => v,
        Err(e) => return domain_error_to_response(e),
    };

    for esc in &created {
        let audit_owner = esc.owner_user_id.clone();
        let _ = state
            .audit_service
            .record(
                Some(esc.request_id.clone()),
                "escalation",
                "auto_create_overdue",
                auth.primary_role_for_audit(),
                Some(auth.sub.to_string()),
                format!("escalation_id={}", esc.id),
                audit_owner,
            )
            .await;
    }

    (
        StatusCode::OK,
        Json(CreatedEscalationsResult {
            created: created.len(),
        }),
    )
        .into_response()
}

#[utoipa::path(
    post,
    path = "/api/v1/technicians",
    tag = "technicians",
    request_body = CreateTechnicianRequest,
    params(
        ("x-role" = Option<String>, Header, description = "supervisor"),
        ("x-actor-id" = Option<String>, Header, description = "Идентификатор актора")
    ),
    responses(
        (status = 201, body = Technician),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse)
    )
)]
async fn create_technician(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateTechnicianRequest>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(&auth, &["admin", "supervisor"]) {
        return r;
    }
    let id = format!("tech-{}", Uuid::new_v4().simple());
    let owner = auth.sub.to_string();
    match state
        .technician_service
        .create(&auth.0, id, body.full_name, body.skills, owner.clone())
        .await
    {
        Ok(item) => {
            let _ = state
                .audit_service
                .record(
                    None,
                    "technician",
                    "create",
                    auth.primary_role_for_audit(),
                    Some(auth.sub.to_string()),
                    format!("technician_id={}", item.id),
                    owner,
                )
                .await;
            (StatusCode::CREATED, Json(item)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/technicians",
    tag = "technicians",
    params(ListQuery),
    responses(
        (status = 200, body = [Technician]),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_technicians(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.technician_service.list(scope).await {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/requests/{id}/audit",
    tag = "audit",
    params(
        ("id" = String, Path, description = "Идентификатор заявки"),
        ListQuery
    ),
    responses(
        (status = 200, body = [AuditRecord]),
        (status = 500, body = ErrorResponse)
    )
)]
async fn list_request_audit(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let scope = auth.data_scope();
    match state.audit_service.list_by_request(&id, scope).await {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/summary",
    tag = "dashboard",
    params(
        ("x-role" = Option<String>, Header, description = "Любая из ролей"),
        ("x-actor-id" = Option<String>, Header, description = "Опционально")
    ),
    responses(
        (status = 200, body = DashboardSummary),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
async fn get_dashboard_summary(
    State(state): State<AppState>,
    auth: JwtAuth,
) -> impl IntoResponse {
    if let Err(r) = require_roles(
        &auth,
        &[
            "admin",
            "supervisor",
            "dispatcher",
            "viewer",
            "technician",
            "user",
        ],
    ) {
        return r;
    }
    let scope = auth.data_scope();
    match state
        .analytics_snapshot_service
        .get_dashboard_summary(now_epoch(), scope)
        .await
    {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/technicians/workload",
    tag = "dashboard",
    params(
        ListQuery,
        ("x-role" = Option<String>, Header, description = "Любая из ролей"),
        ("x-actor-id" = Option<String>, Header, description = "Опционально")
    ),
    responses(
        (status = 200, body = [TechnicianWorkloadSummary]),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
async fn get_technician_workload_summary(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(
        &auth,
        &[
            "admin",
            "supervisor",
            "dispatcher",
            "viewer",
            "technician",
            "user",
        ],
    ) {
        return r;
    }
    let scope = auth.data_scope();
    match state
        .analytics_snapshot_service
        .get_technician_workload_summary(now_epoch(), scope)
        .await
    {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/sla-compliance",
    tag = "dashboard",
    params(
        ("x-role" = Option<String>, Header, description = "Любая из ролей"),
        ("x-actor-id" = Option<String>, Header, description = "Опционально")
    ),
    responses(
        (status = 200, body = SlaComplianceSummary),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
async fn get_sla_compliance_summary(
    State(state): State<AppState>,
    auth: JwtAuth,
) -> impl IntoResponse {
    if let Err(r) = require_roles(
        &auth,
        &[
            "admin",
            "supervisor",
            "dispatcher",
            "viewer",
            "technician",
            "user",
        ],
    ) {
        return r;
    }
    let scope = auth.data_scope();
    match state
        .analytics_snapshot_service
        .get_sla_compliance_summary(now_epoch(), scope)
        .await
    {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/dashboard/sla-compliance-by-priority",
    tag = "dashboard",
    params(
        ListQuery,
        ("x-role" = Option<String>, Header, description = "Любая из ролей"),
        ("x-actor-id" = Option<String>, Header, description = "Опционально")
    ),
    responses(
        (status = 200, body = [SlaComplianceByPriorityItem]),
        (status = 403, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    )
)]
async fn get_sla_compliance_by_priority_summary(
    State(state): State<AppState>,
    auth: JwtAuth,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(r) = require_roles(
        &auth,
        &[
            "admin",
            "supervisor",
            "dispatcher",
            "viewer",
            "technician",
            "user",
        ],
    ) {
        return r;
    }
    let scope = auth.data_scope();
    match state
        .analytics_snapshot_service
        .get_sla_compliance_by_priority_summary(now_epoch(), scope)
        .await
    {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset)))
            .into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Service Processes API",
        version = "0.1.0",
        description = "REST API сервисных заявок. Аутентификация: POST /auth/login, далее заголовок `Authorization: Bearer <JWT>`. Роли в токене: admin, dispatcher, supervisor, technician, viewer, user. Администратор видит все записи; остальные — только сущности со своим owner_user_id. Фоновые задачи: POST /api/v1/jobs (RabbitMQ) и статус в Redis."
    ),
    modifiers(&SecurityAddon),
    paths(
        health,
        login,
        admin_add_role,
        enqueue_job,
        get_job_status,
        create_asset,
        list_assets,
        get_asset,
        create_request,
        list_requests,
        get_request,
        list_overdue_requests,
        update_request_status,
        create_work_order,
        list_work_orders,
        list_work_orders_by_request,
        assign_work_order,
        start_work_order,
        complete_work_order,
        create_escalation,
        list_escalations,
        list_escalations_by_request,
        resolve_escalation,
        escalate_overdue_requests,
        create_technician,
        list_technicians,
        list_request_audit,
        get_dashboard_summary,
        get_technician_workload_summary,
        get_sla_compliance_summary,
        get_sla_compliance_by_priority_summary
    ),
    tags(
        (name = "health", description = "Проверка доступности"),
        (name = "auth", description = "JWT: вход и выдача токена"),
        (name = "admin", description = "Администрирование (RBAC в БД)"),
        (name = "assets", description = "Активы"),
        (name = "requests", description = "Сервисные заявки"),
        (name = "work-orders", description = "Наряды на работы"),
        (name = "escalations", description = "Эскалации"),
        (name = "sla", description = "SLA и автоматические действия"),
        (name = "technicians", description = "Техники"),
        (name = "audit", description = "Журнал аудита"),
        (name = "dashboard", description = "Агрегаты для UI"),
        (name = "jobs", description = "Фоновые задачи (RabbitMQ + Redis)")
    )
)]
pub struct ApiDoc;

fn parse_status(raw: &str) -> Option<RequestStatus> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "new" => Some(RequestStatus::New),
        "planned" => Some(RequestStatus::Planned),
        "in_progress" => Some(RequestStatus::InProgress),
        "resolved" => Some(RequestStatus::Resolved),
        "closed" => Some(RequestStatus::Closed),
        "escalated" => Some(RequestStatus::Escalated),
        _ => None,
    }
}

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn apply_pagination<T>(items: Vec<T>, limit: Option<usize>, offset: Option<usize>) -> Vec<T> {
    let off = offset.unwrap_or(0);
    let lim = limit.unwrap_or(100).min(500);
    items.into_iter().skip(off).take(lim).collect()
}

fn domain_error_to_response(error: DomainError) -> axum::response::Response {
    tracing::warn!(error = %error, "domain error mapped to http response");
    let status = match error {
        DomainError::NotFound(_) => StatusCode::NOT_FOUND,
        DomainError::InvalidTransition => StatusCode::CONFLICT,
        DomainError::EmptyField(_) => StatusCode::BAD_REQUEST,
        DomainError::Forbidden(_) => StatusCode::FORBIDDEN,
        DomainError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
    };

    (
        status,
        Json(ErrorResponse {
            message: error.to_string(),
        }),
    )
        .into_response()
}

#[allow(dead_code)]
fn _assert_send_sync(_items: Vec<ServiceRequest>) {}

#[allow(dead_code)]
fn _assert_send_sync_2(_items: Vec<WorkOrder>) {}

#[allow(dead_code)]
fn _assert_send_sync_3(_items: Vec<Escalation>) {}

#[allow(dead_code)]
fn _assert_send_sync_4(_items: Vec<Technician>) {}

#[allow(dead_code)]
fn _assert_send_sync_5(_items: Vec<AuditRecord>) {}
