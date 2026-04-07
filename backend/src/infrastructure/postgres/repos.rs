use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::domain::analytics::AnalyticsSnapshot;
use crate::domain::entities::{Asset, AuditRecord, Escalation, ServiceRequest, Technician, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{
    AssetState, EscalationState, Priority, RequestStatus, WorkOrderStatus,
};
use crate::infrastructure::postgres::entity::{
    analytics_snapshot, asset, audit_record, escalation, service_request, technician, work_order,
};
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::{
    AnalyticsSnapshotRepository, AssetRepository, AuditRepository, EscalationRepository,
    ServiceRequestRepository, TechnicianRepository, WorkOrderRepository,
};

pub(crate) fn db_err(_: sea_orm::DbErr) -> DomainError {
    DomainError::EmptyField("database")
}

fn asset_state_to_str(s: AssetState) -> &'static str {
    match s {
        AssetState::Active => "Active",
        AssetState::Maintenance => "Maintenance",
        AssetState::Inactive => "Inactive",
    }
}

fn asset_state_from_str(s: &str) -> Result<AssetState, DomainError> {
    match s {
        "Active" => Ok(AssetState::Active),
        "Maintenance" => Ok(AssetState::Maintenance),
        "Inactive" => Ok(AssetState::Inactive),
        _ => Err(DomainError::EmptyField("asset state")),
    }
}

fn priority_to_str(p: Priority) -> &'static str {
    match p {
        Priority::Low => "Low",
        Priority::Medium => "Medium",
        Priority::High => "High",
        Priority::Critical => "Critical",
    }
}

fn priority_from_str(s: &str) -> Result<Priority, DomainError> {
    match s {
        "Low" => Ok(Priority::Low),
        "Medium" => Ok(Priority::Medium),
        "High" => Ok(Priority::High),
        "Critical" => Ok(Priority::Critical),
        _ => Err(DomainError::EmptyField("priority")),
    }
}

fn request_status_to_str(s: RequestStatus) -> &'static str {
    match s {
        RequestStatus::New => "New",
        RequestStatus::Planned => "Planned",
        RequestStatus::InProgress => "InProgress",
        RequestStatus::Resolved => "Resolved",
        RequestStatus::Closed => "Closed",
        RequestStatus::Escalated => "Escalated",
    }
}

fn request_status_from_str(s: &str) -> Result<RequestStatus, DomainError> {
    match s {
        "New" => Ok(RequestStatus::New),
        "Planned" => Ok(RequestStatus::Planned),
        "InProgress" => Ok(RequestStatus::InProgress),
        "Resolved" => Ok(RequestStatus::Resolved),
        "Closed" => Ok(RequestStatus::Closed),
        "Escalated" => Ok(RequestStatus::Escalated),
        _ => Err(DomainError::EmptyField("request status")),
    }
}

fn work_order_status_to_str(s: WorkOrderStatus) -> &'static str {
    match s {
        WorkOrderStatus::Created => "Created",
        WorkOrderStatus::Assigned => "Assigned",
        WorkOrderStatus::InProgress => "InProgress",
        WorkOrderStatus::Completed => "Completed",
        WorkOrderStatus::Cancelled => "Cancelled",
    }
}

fn work_order_status_from_str(s: &str) -> Result<WorkOrderStatus, DomainError> {
    match s {
        "Created" => Ok(WorkOrderStatus::Created),
        "Assigned" => Ok(WorkOrderStatus::Assigned),
        "InProgress" => Ok(WorkOrderStatus::InProgress),
        "Completed" => Ok(WorkOrderStatus::Completed),
        "Cancelled" => Ok(WorkOrderStatus::Cancelled),
        _ => Err(DomainError::EmptyField("work order status")),
    }
}

fn escalation_state_to_str(s: EscalationState) -> &'static str {
    match s {
        EscalationState::Open => "Open",
        EscalationState::Resolved => "Resolved",
    }
}

fn escalation_state_from_str(s: &str) -> Result<EscalationState, DomainError> {
    match s {
        "Open" => Ok(EscalationState::Open),
        "Resolved" => Ok(EscalationState::Resolved),
        _ => Err(DomainError::EmptyField("escalation state")),
    }
}

fn asset_from_model(m: asset::Model) -> Result<Asset, DomainError> {
    Ok(Asset {
        id: m.id,
        kind: m.kind,
        title: m.title,
        location: m.location,
        state: asset_state_from_str(&m.state)?,
        owner_user_id: m.owner_user_id,
    })
}

fn service_request_from_model(m: service_request::Model) -> Result<ServiceRequest, DomainError> {
    Ok(ServiceRequest {
        id: m.id,
        asset_id: m.asset_id,
        description: m.description,
        priority: priority_from_str(&m.priority)?,
        status: request_status_from_str(&m.status)?,
        sla_minutes: m.sla_minutes as u32,
        created_at_epoch_sec: m.created_at_epoch_sec as u64,
        owner_user_id: m.owner_user_id,
    })
}

fn work_order_from_model(m: work_order::Model) -> Result<WorkOrder, DomainError> {
    Ok(WorkOrder {
        id: m.id,
        request_id: m.request_id,
        assignee: m.assignee,
        status: work_order_status_from_str(&m.status)?,
        owner_user_id: m.owner_user_id,
    })
}

fn escalation_from_model(m: escalation::Model) -> Result<Escalation, DomainError> {
    Ok(Escalation {
        id: m.id,
        request_id: m.request_id,
        reason: m.reason,
        state: escalation_state_from_str(&m.state)?,
        owner_user_id: m.owner_user_id,
    })
}

fn technician_from_model(m: technician::Model) -> Result<Technician, DomainError> {
    let skills: Vec<String> = serde_json::from_value(m.skills.clone()).map_err(|_| DomainError::EmptyField("skills"))?;
    Ok(Technician {
        id: m.id,
        full_name: m.full_name,
        skills,
        is_active: m.is_active,
        owner_user_id: m.owner_user_id,
    })
}

fn audit_from_model(m: audit_record::Model) -> Result<AuditRecord, DomainError> {
    Ok(AuditRecord {
        id: m.id,
        request_id: m.request_id,
        entity: m.entity,
        action: m.action,
        actor_role: m.actor_role,
        actor_id: m.actor_id,
        details: m.details,
        created_at_utc: m.created_at_utc,
        owner_user_id: m.owner_user_id,
    })
}

#[derive(Clone)]
pub struct PgAssetRepository {
    db: DatabaseConnection,
}

impl PgAssetRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AssetRepository for PgAssetRepository {
    async fn save(&self, a: Asset) -> Result<(), DomainError> {
        let am = asset::ActiveModel {
            id: Set(a.id),
            kind: Set(a.kind),
            title: Set(a.title),
            location: Set(a.location),
            state: Set(asset_state_to_str(a.state).to_string()),
            owner_user_id: Set(a.owner_user_id),
        };
        asset::Entity::insert(am)
            .on_conflict(
                OnConflict::column(asset::Column::Id)
                    .update_columns([
                        asset::Column::Kind,
                        asset::Column::Title,
                        asset::Column::Location,
                        asset::Column::State,
                        asset::Column::OwnerUserId,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Asset>, DomainError> {
        let mut q = asset::Entity::find_by_id(id);
        if let DataScope::Owner(u) = &scope {
            q = q.filter(asset::Column::OwnerUserId.eq(u.to_string()));
        }
        let row = q.one(&self.db).await.map_err(db_err)?;
        row.map(asset_from_model).transpose()
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<Asset>, DomainError> {
        let mut q = asset::Entity::find();
        if let DataScope::Owner(u) = &scope {
            q = q.filter(asset::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q.order_by_asc(asset::Column::Id).all(&self.db).await.map_err(db_err)?;
        rows.into_iter().map(asset_from_model).collect()
    }
}

#[derive(Clone)]
pub struct PgServiceRequestRepository {
    db: DatabaseConnection,
}

impl PgServiceRequestRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ServiceRequestRepository for PgServiceRequestRepository {
    async fn save(&self, r: ServiceRequest) -> Result<(), DomainError> {
        let am = service_request::ActiveModel {
            id: Set(r.id),
            asset_id: Set(r.asset_id),
            description: Set(r.description),
            priority: Set(priority_to_str(r.priority).to_string()),
            status: Set(request_status_to_str(r.status).to_string()),
            sla_minutes: Set(r.sla_minutes as i32),
            created_at_epoch_sec: Set(r.created_at_epoch_sec as i64),
            owner_user_id: Set(r.owner_user_id),
        };
        service_request::Entity::insert(am)
            .on_conflict(
                OnConflict::column(service_request::Column::Id)
                    .update_columns([
                        service_request::Column::AssetId,
                        service_request::Column::Description,
                        service_request::Column::Priority,
                        service_request::Column::Status,
                        service_request::Column::SlaMinutes,
                        service_request::Column::CreatedAtEpochSec,
                        service_request::Column::OwnerUserId,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<ServiceRequest>, DomainError> {
        let mut q = service_request::Entity::find_by_id(id);
        if let DataScope::Owner(u) = &scope {
            q = q.filter(service_request::Column::OwnerUserId.eq(u.to_string()));
        }
        let row = q.one(&self.db).await.map_err(db_err)?;
        row.map(service_request_from_model).transpose()
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<ServiceRequest>, DomainError> {
        let mut q = service_request::Entity::find();
        if let DataScope::Owner(u) = &scope {
            q = q.filter(service_request::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q
            .order_by_asc(service_request::Column::Id)
            .all(&self.db)
            .await
            .map_err(db_err)?;
        rows.into_iter().map(service_request_from_model).collect()
    }

    async fn update(&self, r: ServiceRequest) -> Result<(), DomainError> {
        self.save(r).await
    }

    async fn list_with_assets(
        &self,
        scope: DataScope,
    ) -> Result<Vec<(ServiceRequest, Option<Asset>)>, DomainError> {
        let mut q = service_request::Entity::find();
        if let DataScope::Owner(u) = &scope {
            q = q.filter(service_request::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q
            .find_also_related(asset::Entity)
            .order_by_asc(service_request::Column::Id)
            .all(&self.db)
            .await
            .map_err(db_err)?;
        let mut out = Vec::with_capacity(rows.len());
        for (req, ast) in rows {
            let req = service_request_from_model(req)?;
            let ast = match ast {
                Some(m) => Some(asset_from_model(m)?),
                None => None,
            };
            out.push((req, ast));
        }
        Ok(out)
    }
}

#[derive(Clone)]
pub struct PgWorkOrderRepository {
    db: DatabaseConnection,
}

impl PgWorkOrderRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl WorkOrderRepository for PgWorkOrderRepository {
    async fn save(&self, w: WorkOrder) -> Result<(), DomainError> {
        let am = work_order::ActiveModel {
            id: Set(w.id),
            request_id: Set(w.request_id),
            assignee: Set(w.assignee),
            status: Set(work_order_status_to_str(w.status).to_string()),
            owner_user_id: Set(w.owner_user_id),
        };
        work_order::Entity::insert(am)
            .on_conflict(
                OnConflict::column(work_order::Column::Id)
                    .update_columns([
                        work_order::Column::RequestId,
                        work_order::Column::Assignee,
                        work_order::Column::Status,
                        work_order::Column::OwnerUserId,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<WorkOrder>, DomainError> {
        let mut q = work_order::Entity::find_by_id(id);
        if let DataScope::Owner(u) = &scope {
            q = q.filter(work_order::Column::OwnerUserId.eq(u.to_string()));
        }
        let row = q.one(&self.db).await.map_err(db_err)?;
        row.map(work_order_from_model).transpose()
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<WorkOrder>, DomainError> {
        let mut q = work_order::Entity::find();
        if let DataScope::Owner(u) = &scope {
            q = q.filter(work_order::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q.order_by_asc(work_order::Column::Id).all(&self.db).await.map_err(db_err)?;
        rows.into_iter().map(work_order_from_model).collect()
    }

    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<WorkOrder>, DomainError> {
        let mut q = work_order::Entity::find().filter(work_order::Column::RequestId.eq(request_id));
        if let DataScope::Owner(u) = &scope {
            q = q.filter(work_order::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q.order_by_asc(work_order::Column::Id).all(&self.db).await.map_err(db_err)?;
        rows.into_iter().map(work_order_from_model).collect()
    }

    async fn update(&self, w: WorkOrder) -> Result<(), DomainError> {
        self.save(w).await
    }
}

#[derive(Clone)]
pub struct PgEscalationRepository {
    db: DatabaseConnection,
}

impl PgEscalationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EscalationRepository for PgEscalationRepository {
    async fn save(&self, e: Escalation) -> Result<(), DomainError> {
        let am = escalation::ActiveModel {
            id: Set(e.id),
            request_id: Set(e.request_id),
            reason: Set(e.reason),
            state: Set(escalation_state_to_str(e.state).to_string()),
            owner_user_id: Set(e.owner_user_id),
        };
        escalation::Entity::insert(am)
            .on_conflict(
                OnConflict::column(escalation::Column::Id)
                    .update_columns([
                        escalation::Column::RequestId,
                        escalation::Column::Reason,
                        escalation::Column::State,
                        escalation::Column::OwnerUserId,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Escalation>, DomainError> {
        let mut q = escalation::Entity::find_by_id(id);
        if let DataScope::Owner(u) = &scope {
            q = q.filter(escalation::Column::OwnerUserId.eq(u.to_string()));
        }
        let row = q.one(&self.db).await.map_err(db_err)?;
        row.map(escalation_from_model).transpose()
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<Escalation>, DomainError> {
        let mut q = escalation::Entity::find();
        if let DataScope::Owner(u) = &scope {
            q = q.filter(escalation::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q.order_by_asc(escalation::Column::Id).all(&self.db).await.map_err(db_err)?;
        rows.into_iter().map(escalation_from_model).collect()
    }

    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<Escalation>, DomainError> {
        let mut q = escalation::Entity::find().filter(escalation::Column::RequestId.eq(request_id));
        if let DataScope::Owner(u) = &scope {
            q = q.filter(escalation::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q.order_by_asc(escalation::Column::Id).all(&self.db).await.map_err(db_err)?;
        rows.into_iter().map(escalation_from_model).collect()
    }

    async fn update(&self, e: Escalation) -> Result<(), DomainError> {
        self.save(e).await
    }
}

#[derive(Clone)]
pub struct PgTechnicianRepository {
    db: DatabaseConnection,
}

impl PgTechnicianRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TechnicianRepository for PgTechnicianRepository {
    async fn save(&self, t: Technician) -> Result<(), DomainError> {
        let skills = serde_json::to_value(&t.skills).map_err(|_| DomainError::EmptyField("skills"))?;
        let am = technician::ActiveModel {
            id: Set(t.id),
            full_name: Set(t.full_name),
            skills: Set(skills),
            is_active: Set(t.is_active),
            owner_user_id: Set(t.owner_user_id),
        };
        technician::Entity::insert(am)
            .on_conflict(
                OnConflict::column(technician::Column::Id)
                    .update_columns([
                        technician::Column::FullName,
                        technician::Column::Skills,
                        technician::Column::IsActive,
                        technician::Column::OwnerUserId,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(db_err)?;
        Ok(())
    }

    async fn get_by_id(&self, id: &str, scope: DataScope) -> Result<Option<Technician>, DomainError> {
        let mut q = technician::Entity::find_by_id(id);
        if let DataScope::Owner(u) = &scope {
            q = q.filter(technician::Column::OwnerUserId.eq(u.to_string()));
        }
        let row = q.one(&self.db).await.map_err(db_err)?;
        row.map(technician_from_model).transpose()
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<Technician>, DomainError> {
        let mut q = technician::Entity::find();
        if let DataScope::Owner(u) = &scope {
            q = q.filter(technician::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q.order_by_asc(technician::Column::Id).all(&self.db).await.map_err(db_err)?;
        rows.into_iter().map(technician_from_model).collect()
    }
}

#[derive(Clone)]
pub struct PgAuditRepository {
    db: DatabaseConnection,
}

impl PgAuditRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuditRepository for PgAuditRepository {
    async fn save(&self, r: AuditRecord) -> Result<(), DomainError> {
        let am = audit_record::ActiveModel {
            id: Set(r.id),
            request_id: Set(r.request_id),
            entity: Set(r.entity),
            action: Set(r.action),
            actor_role: Set(r.actor_role),
            actor_id: Set(r.actor_id),
            details: Set(r.details),
            created_at_utc: Set(r.created_at_utc),
            owner_user_id: Set(r.owner_user_id),
        };
        audit_record::Entity::insert(am).exec(&self.db).await.map_err(db_err)?;
        Ok(())
    }

    async fn list(&self, scope: DataScope) -> Result<Vec<AuditRecord>, DomainError> {
        let mut q = audit_record::Entity::find();
        if let DataScope::Owner(u) = &scope {
            q = q.filter(audit_record::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q
            .order_by_asc(audit_record::Column::CreatedAtUtc)
            .all(&self.db)
            .await
            .map_err(db_err)?;
        rows.into_iter().map(audit_from_model).collect()
    }

    async fn list_by_request(
        &self,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Vec<AuditRecord>, DomainError> {
        let mut q = audit_record::Entity::find().filter(audit_record::Column::RequestId.eq(request_id));
        if let DataScope::Owner(u) = &scope {
            q = q.filter(audit_record::Column::OwnerUserId.eq(u.to_string()));
        }
        let rows = q
            .order_by_asc(audit_record::Column::CreatedAtUtc)
            .all(&self.db)
            .await
            .map_err(db_err)?;
        rows.into_iter().map(audit_from_model).collect()
    }
}

const SNAPSHOT_KEY: &str = "x";

#[derive(Clone)]
pub struct PgAnalyticsSnapshotRepository {
    db: DatabaseConnection,
}

impl PgAnalyticsSnapshotRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AnalyticsSnapshotRepository for PgAnalyticsSnapshotRepository {
    async fn get_latest(&self) -> Result<Option<AnalyticsSnapshot>, DomainError> {
        let row = analytics_snapshot::Entity::find_by_id(SNAPSHOT_KEY)
            .one(&self.db)
            .await
            .map_err(db_err)?;
        let Some(m) = row else {
            return Ok(None);
        };
        let snap: AnalyticsSnapshot = serde_json::from_value(m.payload.clone()).map_err(|_| DomainError::EmptyField("snapshot"))?;
        Ok(Some(snap))
    }

    async fn upsert(&self, snapshot: AnalyticsSnapshot) -> Result<(), DomainError> {
        let payload = serde_json::to_value(&snapshot).map_err(|_| DomainError::EmptyField("snapshot"))?;
        let now = snapshot.updated_at_epoch_sec as i64;
        if let Some(existing) = analytics_snapshot::Entity::find_by_id(SNAPSHOT_KEY)
            .one(&self.db)
            .await
            .map_err(db_err)?
        {
            let mut am: analytics_snapshot::ActiveModel = existing.into();
            am.payload = Set(payload);
            am.updated_at_epoch_sec = Set(now);
            am.update(&self.db).await.map_err(db_err)?;
        } else {
            let am = analytics_snapshot::ActiveModel {
                singleton: Set(SNAPSHOT_KEY.to_string()),
                payload: Set(payload),
                updated_at_epoch_sec: Set(now),
            };
            analytics_snapshot::Entity::insert(am).exec(&self.db).await.map_err(db_err)?;
        }
        Ok(())
    }
}
