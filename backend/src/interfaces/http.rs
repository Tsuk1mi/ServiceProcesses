use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::escalation_service::EscalationAppService;
use crate::application::service_request_service::ServiceRequestAppService;
use crate::application::technician_service::TechnicianAppService;
use crate::application::work_order_service::WorkOrderAppService;
use crate::domain::entities::{Asset, Escalation, ServiceRequest, Technician, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::RequestStatus;
use crate::infrastructure::in_memory::{
    BasicSlaPolicy, InMemoryAssetRepository, InMemoryEscalationRepository, InMemoryRequestRepository,
    InMemoryTechnicianRepository, InMemoryWorkOrderRepository, KeywordPriorityPolicy,
    StdoutEventPublisher,
};
use crate::ports::inbound::{CreateRequestCommand, ServiceRequestUseCase};
use crate::ports::outbound::{AssetRepository, ServiceRequestRepository};

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
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub kind: String,
    pub title: String,
    pub location: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceRequestRequest {
    pub asset_id: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkOrderRequest {
    pub request_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignWorkOrderRequest {
    pub assignee: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEscalationRequest {
    pub request_id: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTechnicianRequest {
    pub full_name: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorRole {
    Dispatcher,
    Technician,
    Supervisor,
    Viewer,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/assets", post(create_asset).get(list_assets))
        .route("/api/v1/assets/:id", get(get_asset))
        .route("/api/v1/requests", post(create_request).get(list_requests))
        .route("/api/v1/requests/:id", get(get_request))
        .route("/api/v1/requests/:id/status", put(update_request_status))
        .route("/api/v1/work-orders", post(create_work_order))
        .route("/api/v1/work-orders/:id/assign", put(assign_work_order))
        .route("/api/v1/work-orders/:id/start", put(start_work_order))
        .route("/api/v1/work-orders/:id/complete", put(complete_work_order))
        .route("/api/v1/requests/:id/work-orders", get(list_work_orders_by_request))
        .route("/api/v1/escalations", post(create_escalation))
        .route("/api/v1/escalations/:id/resolve", put(resolve_escalation))
        .route("/api/v1/requests/:id/escalations", get(list_escalations_by_request))
        .route("/api/v1/technicians", post(create_technician).get(list_technicians))
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

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
            Ok(()) => (StatusCode::CREATED, Json(asset)).into_response(),
            Err(e) => domain_error_to_response(e),
        },
        Err(e) => domain_error_to_response(e),
    }
}

async fn list_assets(State(state): State<AppState>) -> impl IntoResponse {
    match state.assets.list() {
        Ok(assets) => (StatusCode::OK, Json(assets)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn get_asset(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.assets.get_by_id(&id) {
        Ok(Some(asset)) => (StatusCode::OK, Json(asset)).into_response(),
        Ok(None) => domain_error_to_response(DomainError::NotFound("asset")),
        Err(e) => domain_error_to_response(e),
    }
}

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

    match state.service.create_request(command) {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({"result":"created"}))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn list_requests(State(state): State<AppState>) -> impl IntoResponse {
    match state.requests.list() {
        Ok(requests) => (StatusCode::OK, Json(requests)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn get_request(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.requests.get_by_id(&id) {
        Ok(Some(request)) => (StatusCode::OK, Json(request)).into_response(),
        Ok(None) => domain_error_to_response(DomainError::NotFound("service_request")),
        Err(e) => domain_error_to_response(e),
    }
}

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
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"result":"updated"}))).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

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
        Ok(order) => (StatusCode::CREATED, Json(order)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn list_work_orders_by_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.work_order_service.list_by_request(&id) {
        Ok(items) => (StatusCode::OK, Json(items)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

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
        Ok(order) => (StatusCode::OK, Json(order)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn start_work_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = authorize(
        &headers,
        &[ActorRole::Technician, ActorRole::Dispatcher, ActorRole::Supervisor],
    ) {
        return response;
    }
    match state.work_order_service.start(&id) {
        Ok(order) => (StatusCode::OK, Json(order)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn complete_work_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = authorize(
        &headers,
        &[ActorRole::Technician, ActorRole::Dispatcher, ActorRole::Supervisor],
    ) {
        return response;
    }
    match state.work_order_service.complete(&id) {
        Ok(order) => (StatusCode::OK, Json(order)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

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
        Ok(escalation) => (StatusCode::CREATED, Json(escalation)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn list_escalations_by_request(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.escalation_service.list_by_request(&id) {
        Ok(items) => (StatusCode::OK, Json(items)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn resolve_escalation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = authorize(&headers, &[ActorRole::Supervisor, ActorRole::Dispatcher]) {
        return response;
    }
    match state.escalation_service.resolve(&id) {
        Ok(item) => (StatusCode::OK, Json(item)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

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
        Ok(item) => (StatusCode::CREATED, Json(item)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

async fn list_technicians(State(state): State<AppState>) -> impl IntoResponse {
    match state.technician_service.list() {
        Ok(items) => (StatusCode::OK, Json(items)).into_response(),
        Err(e) => domain_error_to_response(e),
    }
}

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

fn authorize(headers: &HeaderMap, allowed: &[ActorRole]) -> Result<(), axum::response::Response> {
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

fn domain_error_to_response(error: DomainError) -> axum::response::Response {
    let status = match error {
        DomainError::NotFound(_) => StatusCode::NOT_FOUND,
        DomainError::InvalidTransition => StatusCode::CONFLICT,
        DomainError::EmptyField(_) => StatusCode::BAD_REQUEST,
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
