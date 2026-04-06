use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

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
use crate::infrastructure::in_memory::{
    BasicSlaPolicy, InMemoryAnalyticsQuery, InMemoryAnalyticsSnapshotRepository, InMemoryAssetRepository,
    InMemoryAuditRepository, InMemoryEscalationRepository, InMemoryRequestRepository,
    InMemoryTechnicianRepository, InMemoryWorkOrderRepository, KeywordPriorityPolicy, StdoutEventPublisher,
};
use crate::ports::inbound::{CreateRequestCommand, ServiceRequestUseCase};
use crate::ports::outbound::{
    AssetRepository, EscalationRepository, ServiceRequestRepository, WorkOrderRepository,
};

type AppService = ServiceRequestAppService<
    InMemoryAssetRepository,
    InMemoryRequestRepository,
    BasicSlaPolicy,
    KeywordPriorityPolicy,
    StdoutEventPublisher,
>;
type WorkOrderService = WorkOrderAppService<
    InMemoryRequestRepository,
    InMemoryWorkOrderRepository,
    InMemoryTechnicianRepository,
    StdoutEventPublisher,
>;
type EscalationService = EscalationAppService<
    InMemoryRequestRepository,
    InMemoryEscalationRepository,
    StdoutEventPublisher,
>;
type TechnicianService = TechnicianAppService<InMemoryTechnicianRepository>;
type AuditService = AuditAppService<InMemoryAuditRepository>;
type SlaService =
    SlaAppService<InMemoryRequestRepository, InMemoryEscalationRepository, StdoutEventPublisher>;
type ReportingService = ReportingAppService<InMemoryAnalyticsQuery>;
type AnalyticsSnapshotService =
    AnalyticsSnapshotAppService<InMemoryAnalyticsQuery, InMemoryAnalyticsSnapshotRepository>;

#[derive(Clone)]
pub struct AppState {
    pub assets: InMemoryAssetRepository,
    pub requests: InMemoryRequestRepository,
    pub service: AppService,
    pub work_orders: InMemoryWorkOrderRepository,
    pub work_order_service: WorkOrderService,
    pub escalations: InMemoryEscalationRepository,
    pub escalation_service: EscalationService,
    pub technicians: InMemoryTechnicianRepository,
    pub technician_service: TechnicianService,
    pub audit: InMemoryAuditRepository,
    pub audit_service: AuditService,
    pub sla_service: SlaService,
    pub reporting_service: ReportingService,
    pub analytics_snapshot_service: AnalyticsSnapshotService,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorRole {
    Dispatcher,
    Technician,
    Supervisor,
    Viewer,
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
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
        .with_state(state);

    Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .merge(api)
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
    headers: HeaderMap,
    Json(body): Json<CreateAssetRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Dispatcher, ActorRole::Supervisor]) {
        return response;
    }
    let id = format!("asset-{}", Uuid::new_v4().simple());
    match Asset::new(id, body.kind, body.title, body.location) {
        Ok(asset) => match state.assets.save(asset.clone()) {
            Ok(()) => {
                let _ = state.audit_service.record(
                    None,
                    "asset",
                    "create",
                    role_name(parse_role(&headers)),
                    parse_actor_id(&headers),
                    format!("asset_id={}", asset.id),
                );
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
async fn list_assets(State(state): State<AppState>) -> impl IntoResponse {
    match state.assets.list() {
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
async fn get_asset(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.assets.get_by_id(&id) {
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
    headers: HeaderMap,
    Json(body): Json<CreateServiceRequestRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Dispatcher, ActorRole::Supervisor]) {
        return response;
    }
    let request_id = format!("req-{}", Uuid::new_v4().simple());
    let command = CreateRequestCommand {
        request_id,
        asset_id: body.asset_id,
        description: body.description,
    };

    match state.service.create_request(command.clone()) {
        Ok(()) => {
            let _ = state.audit_service.record(
                Some(command.request_id.clone()),
                "service_request",
                "create",
                role_name(parse_role(&headers)),
                parse_actor_id(&headers),
                format!("asset_id={}", command.asset_id),
            );
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
    Query(query): Query<RequestListQuery>,
) -> impl IntoResponse {
    match state.requests.list() {
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
async fn get_request(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.requests.get_by_id(&id) {
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
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state.sla_service.list_overdue_requests(now_epoch()) {
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
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Dispatcher, ActorRole::Supervisor]) {
        return response;
    }
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

    match state.service.update_status(&id, next) {
        Ok(()) => {
            let _ = state.audit_service.record(
                Some(id.clone()),
                "service_request",
                "update_status",
                role_name(parse_role(&headers)),
                parse_actor_id(&headers),
                format!("status={:?}", next),
            );
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
    headers: HeaderMap,
    Json(body): Json<CreateWorkOrderRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Dispatcher, ActorRole::Supervisor]) {
        return response;
    }
    let id = format!("wo-{}", Uuid::new_v4().simple());
    match state
        .work_order_service
        .create_work_order(id, body.request_id)
    {
        Ok(order) => {
            let _ = state.audit_service.record(
                Some(order.request_id.clone()),
                "work_order",
                "create",
                role_name(parse_role(&headers)),
                parse_actor_id(&headers),
                format!("work_order_id={}", order.id),
            );
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
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(response) = authorize(
        &headers,
        &[
            ActorRole::Supervisor,
            ActorRole::Dispatcher,
            ActorRole::Viewer,
            ActorRole::Technician,
        ],
    ) {
        return response;
    }
    match state.work_orders.list() {
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
    Path(id): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state.work_order_service.list_by_request(&id) {
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
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<AssignWorkOrderRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Dispatcher, ActorRole::Supervisor]) {
        return response;
    }
    match state.work_order_service.assign(&id, body.assignee) {
        Ok(order) => {
            let _ = state.audit_service.record(
                Some(order.request_id.clone()),
                "work_order",
                "assign",
                role_name(parse_role(&headers)),
                parse_actor_id(&headers),
                format!("work_order_id={},assignee={}", order.id, order.assignee.clone().unwrap_or_default()),
            );
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
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let role = match authorize(
        &headers,
        &[ActorRole::Technician, ActorRole::Dispatcher, ActorRole::Supervisor],
    ) {
        Ok(role) => role,
        Err(response) => {
            return response;
        }
    };
    if role == ActorRole::Technician {
        let actor_id = match parse_actor_id(&headers) {
            Some(v) => v,
            None => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        message: "x-actor-id is required for technician role".to_string(),
                    }),
                )
                    .into_response()
            }
        };
        match state.work_order_service.start_by_actor(&id, &actor_id) {
            Ok(order) => {
                let _ = state.audit_service.record(
                    Some(order.request_id.clone()),
                    "work_order",
                    "start",
                    role_name(Some(role)),
                    Some(actor_id),
                    format!("work_order_id={}", order.id),
                );
                (StatusCode::OK, Json(order)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        }
    } else {
        match state.work_order_service.start(&id) {
            Ok(order) => {
                let _ = state.audit_service.record(
                    Some(order.request_id.clone()),
                    "work_order",
                    "start",
                    role_name(Some(role)),
                    parse_actor_id(&headers),
                    format!("work_order_id={}", order.id),
                );
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
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let role = match authorize(
        &headers,
        &[ActorRole::Technician, ActorRole::Dispatcher, ActorRole::Supervisor],
    ) {
        Ok(role) => role,
        Err(response) => {
            return response;
        }
    };
    if role == ActorRole::Technician {
        let actor_id = match parse_actor_id(&headers) {
            Some(v) => v,
            None => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        message: "x-actor-id is required for technician role".to_string(),
                    }),
                )
                    .into_response()
            }
        };
        match state.work_order_service.complete_by_actor(&id, &actor_id) {
            Ok(order) => {
                let _ = state.audit_service.record(
                    Some(order.request_id.clone()),
                    "work_order",
                    "complete",
                    role_name(Some(role)),
                    Some(actor_id),
                    format!("work_order_id={}", order.id),
                );
                (StatusCode::OK, Json(order)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        }
    } else {
        match state.work_order_service.complete(&id) {
            Ok(order) => {
                let _ = state.audit_service.record(
                    Some(order.request_id.clone()),
                    "work_order",
                    "complete",
                    role_name(Some(role)),
                    parse_actor_id(&headers),
                    format!("work_order_id={}", order.id),
                );
                (StatusCode::OK, Json(order)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        }
    }
}

fn parse_actor_id(headers: &HeaderMap) -> Option<String> {
    Some(headers.get("x-actor-id")?.to_str().ok()?.trim().to_string())
}

fn authorize(headers: &HeaderMap, allowed: &[ActorRole]) -> Result<ActorRole, axum::response::Response> {
    let role = parse_role(headers).ok_or_else(|| {
        (
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                message: "missing or invalid x-role header".to_string(),
            }),
        )
            .into_response()
    })?;

    if allowed.contains(&role) {
        return Ok(role);
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
    headers: HeaderMap,
    Json(body): Json<CreateEscalationRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Dispatcher, ActorRole::Supervisor]) {
        return response;
    }
    let id = format!("esc-{}", Uuid::new_v4().simple());
    match state
        .escalation_service
        .create_escalation(id, body.request_id, body.reason)
    {
        Ok(escalation) => {
            let _ = state.audit_service.record(
                Some(escalation.request_id.clone()),
                "escalation",
                "create",
                role_name(parse_role(&headers)),
                parse_actor_id(&headers),
                format!("escalation_id={}", escalation.id),
            );
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
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(response) = authorize(
        &headers,
        &[
            ActorRole::Supervisor,
            ActorRole::Dispatcher,
            ActorRole::Viewer,
            ActorRole::Technician,
        ],
    ) {
        return response;
    }
    match state.escalations.list() {
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
    Path(id): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state.escalation_service.list_by_request(&id) {
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
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Supervisor, ActorRole::Dispatcher]) {
        return response;
    }
    match state.escalation_service.resolve(&id) {
        Ok(item) => {
            let _ = state.audit_service.record(
                Some(item.request_id.clone()),
                "escalation",
                "resolve",
                role_name(parse_role(&headers)),
                parse_actor_id(&headers),
                format!("escalation_id={}", item.id),
            );
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
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Supervisor, ActorRole::Dispatcher]) {
        return response;
    }
    let created = match state
        .sla_service
        .auto_escalate_overdue(now_epoch(), "Automatic SLA overdue escalation")
    {
        Ok(v) => v,
        Err(e) => return domain_error_to_response(e),
    };

    for esc in &created {
        let _ = state.audit_service.record(
            Some(esc.request_id.clone()),
            "escalation",
            "auto_create_overdue",
            role_name(parse_role(&headers)),
            parse_actor_id(&headers),
            format!("escalation_id={}", esc.id),
        );
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
    headers: HeaderMap,
    Json(body): Json<CreateTechnicianRequest>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Supervisor]) {
        return response;
    }
    let id = format!("tech-{}", Uuid::new_v4().simple());
    match state
        .technician_service
        .create(id, body.full_name, body.skills)
    {
        Ok(item) => {
            let _ = state.audit_service.record(
                None,
                "technician",
                "create",
                role_name(parse_role(&headers)),
                parse_actor_id(&headers),
                format!("technician_id={}", item.id),
            );
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
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state.technician_service.list() {
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
    Path(id): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    match state.audit_service.list_by_request(&id) {
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
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = authorize(
        &headers,
        &[
            ActorRole::Supervisor,
            ActorRole::Dispatcher,
            ActorRole::Viewer,
            ActorRole::Technician,
        ],
    ) {
        return response;
    }
    match state
        .analytics_snapshot_service
        .get_dashboard_summary(now_epoch())
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
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(response) = authorize(
        &headers,
        &[
            ActorRole::Supervisor,
            ActorRole::Dispatcher,
            ActorRole::Viewer,
            ActorRole::Technician,
        ],
    ) {
        return response;
    }
    match state
        .analytics_snapshot_service
        .get_technician_workload_summary(now_epoch())
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
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(response) = authorize(
        &headers,
        &[
            ActorRole::Supervisor,
            ActorRole::Dispatcher,
            ActorRole::Viewer,
            ActorRole::Technician,
        ],
    ) {
        return response;
    }
    match state
        .analytics_snapshot_service
        .get_sla_compliance_summary(now_epoch())
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
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    if let Err(response) = authorize(
        &headers,
        &[
            ActorRole::Supervisor,
            ActorRole::Dispatcher,
            ActorRole::Viewer,
            ActorRole::Technician,
        ],
    ) {
        return response;
    }
    match state
        .analytics_snapshot_service
        .get_sla_compliance_by_priority_summary(now_epoch())
    {
        Ok(items) => (StatusCode::OK, Json(apply_pagination(items, query.limit, query.offset)))
            .into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Service Processes API",
        version = "0.1.0",
        description = "REST API сервисных заявок: активы, заявки, наряды, эскалации, SLA и дашборды. Для большинства операций нужен заголовок `x-role` (dispatcher, technician, supervisor, viewer); для роли technician при ряде операций обязателен `x-actor-id`."
    ),
    paths(
        health,
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
        (name = "assets", description = "Активы"),
        (name = "requests", description = "Сервисные заявки"),
        (name = "work-orders", description = "Наряды на работы"),
        (name = "escalations", description = "Эскалации"),
        (name = "sla", description = "SLA и автоматические действия"),
        (name = "technicians", description = "Техники"),
        (name = "audit", description = "Журнал аудита"),
        (name = "dashboard", description = "Агрегаты для UI")
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

fn parse_role(headers: &HeaderMap) -> Option<ActorRole> {
    let raw = headers.get("x-role")?.to_str().ok()?.trim().to_ascii_lowercase();
    match raw.as_str() {
        "dispatcher" => Some(ActorRole::Dispatcher),
        "technician" => Some(ActorRole::Technician),
        "supervisor" => Some(ActorRole::Supervisor),
        "viewer" => Some(ActorRole::Viewer),
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

fn role_name(role: Option<ActorRole>) -> &'static str {
    match role {
        Some(ActorRole::Dispatcher) => "dispatcher",
        Some(ActorRole::Technician) => "technician",
        Some(ActorRole::Supervisor) => "supervisor",
        Some(ActorRole::Viewer) => "viewer",
        None => "unknown",
    }
}

fn domain_error_to_response(error: DomainError) -> axum::response::Response {
    let status = match error {
        DomainError::NotFound(_) => StatusCode::NOT_FOUND,
        DomainError::InvalidTransition => StatusCode::CONFLICT,
        DomainError::EmptyField(_) => StatusCode::BAD_REQUEST,
        DomainError::Forbidden(_) => StatusCode::FORBIDDEN,
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
