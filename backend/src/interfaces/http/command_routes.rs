use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{post, put};
use axum::{Json, Router};
use uuid::Uuid;

use crate::ports::inbound::{CreateRequestCommand, ServiceRequestUseCase};

use super::security::{validate_required_text, validate_string_enum, validate_string_list};
use super::{
    domain_error_to_response, parse_status, AppState, AssignWorkOrderRequest, CreateAssetRequest,
    CreateEscalationRequest, CreateServiceRequestRequest, CreateTechnicianRequest, CreateWorkOrderRequest,
    JwtAuth, MutationResult, UpdateStatusRequest,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/commands/assets", post(create_asset_command))
        .route("/api/v1/commands/requests", post(create_request_command))
        .route("/api/v1/commands/requests/:id/status", put(update_request_status_command))
        .route("/api/v1/commands/work-orders", post(create_work_order_command))
        .route("/api/v1/commands/work-orders/:id/assign", put(assign_work_order_command))
        .route("/api/v1/commands/escalations", post(create_escalation_command))
        .route("/api/v1/commands/technicians", post(create_technician_command))
}

async fn create_asset_command(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateAssetRequest>,
) -> impl IntoResponse {
    if let Err(r) = super::require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    if let Err(e) = validate_required_text("kind", &body.kind, 120) {
        return domain_error_to_response(e);
    }
    if let Err(e) = validate_required_text("title", &body.title, 255) {
        return domain_error_to_response(e);
    }
    if let Err(e) = validate_required_text("location", &body.location, 255) {
        return domain_error_to_response(e);
    }
    let id = format!("asset-{}", Uuid::new_v4().simple());
    let owner = auth.sub.to_string();
    match crate::domain::entities::Asset::new(id, body.kind, body.title, body.location, owner) {
        Ok(asset) => match state.assets.save(asset.clone(), auth.data_scope()).await {
            Ok(()) => {
                state.metrics.record_command("create_asset");
                (axum::http::StatusCode::CREATED, Json(asset)).into_response()
            }
            Err(e) => domain_error_to_response(e),
        },
        Err(e) => domain_error_to_response(e),
    }
}

async fn create_request_command(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateServiceRequestRequest>,
) -> impl IntoResponse {
    if let Err(r) = super::require_roles(&auth, &["admin", "dispatcher", "supervisor", "user"]) {
        return r;
    }
    if let Err(e) = validate_required_text("asset_id", &body.asset_id, 128) {
        return domain_error_to_response(e);
    }
    if let Err(e) = validate_required_text("description", &body.description, 2000) {
        return domain_error_to_response(e);
    }
    let command = CreateRequestCommand {
        request_id: format!("req-{}", Uuid::new_v4().simple()),
        asset_id: body.asset_id,
        description: body.description,
    };
    match state
        .service
        .create_request(&auth.0, command, auth.data_scope())
        .await
    {
        Ok(()) => {
            state.metrics.record_command("create_request");
            (
                axum::http::StatusCode::ACCEPTED,
                Json(MutationResult {
                    result: "accepted".to_string(),
                }),
            )
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn update_request_status_command(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
    Json(body): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    if let Err(r) = super::require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    if let Err(e) = validate_string_enum(
        "status",
        &body.status,
        &["new", "planned", "in_progress", "resolved", "closed", "escalated"],
    ) {
        return domain_error_to_response(e);
    }
    let Some(status) = parse_status(&body.status) else {
        return domain_error_to_response(crate::domain::errors::DomainError::InvalidInput("status"));
    };
    match state
        .service
        .update_status(&auth.0, &id, status, auth.data_scope())
        .await
    {
        Ok(()) => {
            state.metrics.record_command("update_request_status");
            (axum::http::StatusCode::ACCEPTED, Json(MutationResult { result: "accepted".into() }))
                .into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn create_work_order_command(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateWorkOrderRequest>,
) -> impl IntoResponse {
    if let Err(r) = super::require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    if let Err(e) = validate_required_text("request_id", &body.request_id, 128) {
        return domain_error_to_response(e);
    }
    match state
        .work_order_service
        .create_work_order(
            &auth.0,
            format!("wo-{}", Uuid::new_v4().simple()),
            body.request_id,
            auth.data_scope(),
        )
        .await
    {
        Ok(order) => {
            state.metrics.record_command("create_work_order");
            (axum::http::StatusCode::ACCEPTED, Json(order)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn assign_work_order_command(
    State(state): State<AppState>,
    auth: JwtAuth,
    Path(id): Path<String>,
    Json(body): Json<AssignWorkOrderRequest>,
) -> impl IntoResponse {
    if let Err(r) = super::require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    if let Err(e) = validate_required_text("assignee", &body.assignee, 128) {
        return domain_error_to_response(e);
    }
    match state
        .work_order_service
        .assign(&auth.0, &id, body.assignee, auth.data_scope())
        .await
    {
        Ok(order) => {
            state.metrics.record_command("assign_work_order");
            (axum::http::StatusCode::ACCEPTED, Json(order)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn create_escalation_command(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateEscalationRequest>,
) -> impl IntoResponse {
    if let Err(r) = super::require_roles(&auth, &["admin", "dispatcher", "supervisor"]) {
        return r;
    }
    if let Err(e) = validate_required_text("request_id", &body.request_id, 128) {
        return domain_error_to_response(e);
    }
    if let Err(e) = validate_required_text("reason", &body.reason, 1024) {
        return domain_error_to_response(e);
    }
    match state
        .escalation_service
        .create_escalation(
            &auth.0,
            format!("esc-{}", Uuid::new_v4().simple()),
            body.request_id,
            body.reason,
            auth.data_scope(),
        )
        .await
    {
        Ok(item) => {
            state.metrics.record_command("create_escalation");
            (axum::http::StatusCode::ACCEPTED, Json(item)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}

async fn create_technician_command(
    State(state): State<AppState>,
    auth: JwtAuth,
    Json(body): Json<CreateTechnicianRequest>,
) -> impl IntoResponse {
    if let Err(r) = super::require_roles(&auth, &["admin", "supervisor"]) {
        return r;
    }
    if let Err(e) = validate_required_text("full_name", &body.full_name, 255) {
        return domain_error_to_response(e);
    }
    if let Err(e) = validate_string_list("skills", &body.skills, 80) {
        return domain_error_to_response(e);
    }
    match state
        .technician_service
        .create(
            &auth.0,
            format!("tech-{}", Uuid::new_v4().simple()),
            body.full_name,
            body.skills,
            auth.sub.to_string(),
        )
        .await
    {
        Ok(item) => {
            state.metrics.record_command("create_technician");
            (axum::http::StatusCode::ACCEPTED, Json(item)).into_response()
        }
        Err(e) => domain_error_to_response(e),
    }
}
