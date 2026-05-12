use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement, Value};

use crate::application::analytics_snapshot_service::AnalyticsSnapshotAppService;
use crate::domain::analytics::{
    AnalyticsSnapshot, DashboardSummary, SlaComplianceByPriorityItem, SlaComplianceSummary,
    TechnicianWorkloadSummary,
};
use crate::domain::entities::{Asset, Escalation, ServiceRequest, Technician, WorkOrder};
use crate::domain::errors::DomainError;
use crate::domain::value_objects::{EscalationState, RequestStatus, WorkOrderStatus};
use crate::ports::data_scope::DataScope;
use crate::ports::outbound::{
    AssetRepository, EscalationRepository, ServiceRequestRepository, TechnicianRepository, WorkOrderRepository,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestListFilter {
    pub limit: usize,
    pub offset: usize,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub overdue_only: bool,
}

impl Default for RequestListFilter {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
            status: None,
            priority: None,
            overdue_only: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadRequestListItem {
    pub request_id: String,
    pub asset_id: String,
    pub asset_title: String,
    pub asset_location: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub assignee: Option<String>,
    pub open_escalation_count: usize,
    pub work_order_count: usize,
    pub overdue: bool,
    pub sla_deadline_epoch_sec: u64,
    pub created_at_epoch_sec: u64,
    pub owner_user_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReadRequestDetail {
    pub request_id: String,
    pub asset_id: String,
    pub asset_title: String,
    pub asset_location: String,
    pub description: String,
    pub priority: String,
    pub status: String,
    pub assignee: Option<String>,
    pub work_order_status: Option<String>,
    pub open_escalation_count: usize,
    pub latest_escalation_reason: Option<String>,
    pub overdue: bool,
    pub sla_deadline_epoch_sec: u64,
    pub created_at_epoch_sec: u64,
    pub owner_user_id: String,
}

#[async_trait]
pub trait ReadModelQueryPort: Send + Sync {
    async fn list_requests(
        &self,
        now_epoch: u64,
        scope: DataScope,
        filter: RequestListFilter,
    ) -> Result<Vec<ReadRequestListItem>, DomainError>;

    async fn get_request_detail(
        &self,
        now_epoch: u64,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Option<ReadRequestDetail>, DomainError>;

    async fn dashboard_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<DashboardSummary, DomainError>;

    async fn sla_compliance_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<SlaComplianceSummary, DomainError>;

    async fn sla_compliance_by_priority(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<SlaComplianceByPriorityItem>, DomainError>;

    async fn technician_workload(
        &self,
        scope: DataScope,
    ) -> Result<Vec<TechnicianWorkloadSummary>, DomainError>;
}

#[derive(Clone)]
pub struct InMemoryReadModelQuery {
    pub assets: Arc<dyn AssetRepository>,
    pub requests: Arc<dyn ServiceRequestRepository>,
    pub work_orders: Arc<dyn WorkOrderRepository>,
    pub escalations: Arc<dyn EscalationRepository>,
    pub technicians: Arc<dyn TechnicianRepository>,
    pub analytics_snapshot_service: AnalyticsSnapshotAppService,
}

#[async_trait]
impl ReadModelQueryPort for InMemoryReadModelQuery {
    async fn list_requests(
        &self,
        now_epoch: u64,
        scope: DataScope,
        filter: RequestListFilter,
    ) -> Result<Vec<ReadRequestListItem>, DomainError> {
        let assets = self
            .assets
            .list(scope)
            .await?
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect::<HashMap<_, _>>();
        let work_orders = self.work_orders.list(DataScope::All).await?;
        let escalations = self.escalations.list(DataScope::All).await?;
        let mut items = self
            .requests
            .list(match scope {
                DataScope::All => DataScope::All,
                DataScope::Owner(user_id) => DataScope::Owner(user_id),
            })
            .await?
            .into_iter()
            .filter_map(|request| request_to_list_item(now_epoch, &assets, &work_orders, &escalations, request).ok())
            .collect::<Vec<_>>();

        apply_filter(&mut items, filter);
        Ok(items)
    }

    async fn get_request_detail(
        &self,
        now_epoch: u64,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Option<ReadRequestDetail>, DomainError> {
        let assets = self
            .assets
            .list(scope)
            .await?
            .into_iter()
            .map(|item| (item.id.clone(), item))
            .collect::<HashMap<_, _>>();
        let request = self.requests.get_by_id(request_id, scope).await?;
        let Some(request) = request else {
            return Ok(None);
        };
        let work_orders = self.work_orders.list_by_request(request_id, DataScope::All).await?;
        let escalations = self.escalations.list_by_request(request_id, DataScope::All).await?;
        let asset = assets
            .get(&request.asset_id)
            .ok_or(DomainError::NotFound("asset"))?;
        Ok(Some(build_detail(now_epoch, asset, request, &work_orders, &escalations)))
    }

    async fn dashboard_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<DashboardSummary, DomainError> {
        self.analytics_snapshot_service
            .get_dashboard_summary(now_epoch, scope)
            .await
    }

    async fn sla_compliance_summary(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<SlaComplianceSummary, DomainError> {
        self.analytics_snapshot_service
            .get_sla_compliance_summary(now_epoch, scope)
            .await
    }

    async fn sla_compliance_by_priority(
        &self,
        now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<SlaComplianceByPriorityItem>, DomainError> {
        self.analytics_snapshot_service
            .get_sla_compliance_by_priority_summary(now_epoch, scope)
            .await
    }

    async fn technician_workload(&self, scope: DataScope) -> Result<Vec<TechnicianWorkloadSummary>, DomainError> {
        self.analytics_snapshot_service
            .get_technician_workload_summary(now_epoch(), scope)
            .await
    }
}

#[derive(Clone)]
pub struct PgReadModelQuery {
    db: DatabaseConnection,
}

impl PgReadModelQuery {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn all_request_items(&self, scope: DataScope) -> Result<Vec<ReadRequestListItem>, DomainError> {
        let mut sql = String::from(
            "SELECT request_id, asset_id, asset_title, asset_location, description, priority, status, \
             assignee, open_escalation_count, work_order_count, overdue, sla_deadline_epoch_sec, \
             created_at_epoch_sec, owner_user_id FROM read_request_list_item WHERE 1=1",
        );
        let mut values = Vec::<Value>::new();
        append_scope(&mut sql, &mut values, scope);
        sql.push_str(" ORDER BY created_at_epoch_sec DESC, request_id ASC");
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
            .await
            .map_err(|_| DomainError::EmptyField("database"))?;
        rows.into_iter().map(row_to_request_list_item).collect()
    }

    async fn analytics_snapshot(&self) -> Result<Option<AnalyticsSnapshot>, DomainError> {
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT payload FROM analytics_snapshot WHERE singleton = $1",
            vec!["x".into()],
        );
        let row = self
            .db
            .query_one(stmt)
            .await
            .map_err(|_| DomainError::EmptyField("database"))?;
        let Some(row) = row else {
            return Ok(None);
        };
        let payload: serde_json::Value = row.try_get("", "payload").map_err(|_| DomainError::EmptyField("snapshot"))?;
        let snapshot = serde_json::from_value(payload).map_err(|_| DomainError::EmptyField("snapshot"))?;
        Ok(Some(snapshot))
    }
}

#[async_trait]
impl ReadModelQueryPort for PgReadModelQuery {
    async fn list_requests(
        &self,
        _now_epoch: u64,
        scope: DataScope,
        filter: RequestListFilter,
    ) -> Result<Vec<ReadRequestListItem>, DomainError> {
        let mut sql = String::from(
            "SELECT request_id, asset_id, asset_title, asset_location, description, priority, status, \
             assignee, open_escalation_count, work_order_count, overdue, sla_deadline_epoch_sec, \
             created_at_epoch_sec, owner_user_id FROM read_request_list_item WHERE 1=1",
        );
        let mut values = Vec::<Value>::new();
        append_scope(&mut sql, &mut values, scope);
        if let Some(status) = filter.status.clone() {
            values.push(status.into());
            sql.push_str(&format!(" AND status = ${}", values.len()));
        }
        if let Some(priority) = filter.priority.clone() {
            values.push(priority.into());
            sql.push_str(&format!(" AND priority = ${}", values.len()));
        }
        if filter.overdue_only {
            sql.push_str(" AND overdue = TRUE");
        }
        values.push((filter.limit as i64).into());
        sql.push_str(&format!(" ORDER BY created_at_epoch_sec DESC, request_id ASC LIMIT ${}", values.len()));
        values.push((filter.offset as i64).into());
        sql.push_str(&format!(" OFFSET ${}", values.len()));

        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
            .await
            .map_err(|_| DomainError::EmptyField("database"))?;
        rows.into_iter().map(row_to_request_list_item).collect()
    }

    async fn get_request_detail(
        &self,
        _now_epoch: u64,
        request_id: &str,
        scope: DataScope,
    ) -> Result<Option<ReadRequestDetail>, DomainError> {
        let mut sql = String::from(
            "SELECT request_id, asset_id, asset_title, asset_location, description, priority, status, \
             assignee, work_order_status, open_escalation_count, latest_escalation_reason, overdue, \
             sla_deadline_epoch_sec, created_at_epoch_sec, owner_user_id \
             FROM read_request_detail WHERE request_id = $1",
        );
        let mut values = vec![request_id.to_string().into()];
        append_scope(&mut sql, &mut values, scope);
        let row = self
            .db
            .query_one(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
            .await
            .map_err(|_| DomainError::EmptyField("database"))?;
        row.map(row_to_request_detail).transpose()
    }

    async fn dashboard_summary(
        &self,
        _now_epoch: u64,
        scope: DataScope,
    ) -> Result<DashboardSummary, DomainError> {
        if scope.is_all() {
            if let Some(snapshot) = self.analytics_snapshot().await? {
                return Ok(snapshot.dashboard);
            }
        }
        let items = self.all_request_items(scope).await?;
        let total_work_orders = self.technician_workload(scope).await?.into_iter().map(|item| item.total).sum();
        let active_work_orders = self
            .technician_workload(scope)
            .await?
            .into_iter()
            .map(|item| item.assigned + item.in_progress)
            .sum();
        Ok(DashboardSummary {
            total_requests: items.len(),
            open_requests: items.iter().filter(|item| !is_terminal_status(&item.status)).count(),
            in_progress_requests: items.iter().filter(|item| item.status == "InProgress").count(),
            resolved_requests: items.iter().filter(|item| item.status == "Resolved").count(),
            closed_requests: items.iter().filter(|item| item.status == "Closed").count(),
            overdue_requests: items
                .iter()
                .filter(|item| item.overdue && !is_terminal_status(&item.status))
                .count(),
            total_work_orders,
            active_work_orders,
            open_escalations: items.iter().map(|item| item.open_escalation_count).sum(),
        })
    }

    async fn sla_compliance_summary(
        &self,
        _now_epoch: u64,
        scope: DataScope,
    ) -> Result<SlaComplianceSummary, DomainError> {
        if scope.is_all() {
            if let Some(snapshot) = self.analytics_snapshot().await? {
                return Ok(snapshot.sla_compliance);
            }
        }
        let items = self.all_request_items(scope).await?;
        Ok(compute_sla_summary(&items))
    }

    async fn sla_compliance_by_priority(
        &self,
        _now_epoch: u64,
        scope: DataScope,
    ) -> Result<Vec<SlaComplianceByPriorityItem>, DomainError> {
        if scope.is_all() {
            if let Some(snapshot) = self.analytics_snapshot().await? {
                return Ok(snapshot.sla_compliance_by_priority);
            }
        }
        let items = self.all_request_items(scope).await?;
        Ok(compute_sla_by_priority(&items))
    }

    async fn technician_workload(&self, scope: DataScope) -> Result<Vec<TechnicianWorkloadSummary>, DomainError> {
        let mut sql = String::from(
            "SELECT COALESCE(assignee, '') AS technician_id, COALESCE(assignee_name, 'Unassigned') AS full_name, \
             SUM(CASE WHEN status = 'Assigned' THEN 1 ELSE 0 END) AS assigned, \
             SUM(CASE WHEN status = 'InProgress' THEN 1 ELSE 0 END) AS in_progress, \
             SUM(CASE WHEN status = 'Completed' THEN 1 ELSE 0 END) AS completed, \
             COUNT(*) AS total \
             FROM read_work_order_item WHERE 1=1",
        );
        let mut values = Vec::<Value>::new();
        append_scope(&mut sql, &mut values, scope);
        sql.push_str(" GROUP BY assignee, assignee_name ORDER BY total DESC, technician_id ASC");
        let rows = self
            .db
            .query_all(Statement::from_sql_and_values(DbBackend::Postgres, sql, values))
            .await
            .map_err(|_| DomainError::EmptyField("database"))?;
        rows.into_iter()
            .map(|row| {
                Ok(TechnicianWorkloadSummary {
                    technician_id: row
                        .try_get::<String>("", "technician_id")
                        .map_err(|_| DomainError::EmptyField("technician_id"))?,
                    full_name: row
                        .try_get::<String>("", "full_name")
                        .map_err(|_| DomainError::EmptyField("full_name"))?,
                    assigned: row
                        .try_get::<i64>("", "assigned")
                        .map_err(|_| DomainError::EmptyField("assigned"))? as usize,
                    in_progress: row
                        .try_get::<i64>("", "in_progress")
                        .map_err(|_| DomainError::EmptyField("in_progress"))? as usize,
                    completed: row
                        .try_get::<i64>("", "completed")
                        .map_err(|_| DomainError::EmptyField("completed"))? as usize,
                    total: row
                        .try_get::<i64>("", "total")
                        .map_err(|_| DomainError::EmptyField("total"))? as usize,
                })
            })
            .collect()
    }
}

fn append_scope(sql: &mut String, values: &mut Vec<Value>, scope: DataScope) {
    if let DataScope::Owner(user_id) = scope {
        values.push(user_id.to_string().into());
        sql.push_str(&format!(" AND owner_user_id = ${}", values.len()));
    }
}

fn row_to_request_list_item(row: sea_orm::QueryResult) -> Result<ReadRequestListItem, DomainError> {
    Ok(ReadRequestListItem {
        request_id: row.try_get("", "request_id").map_err(|_| DomainError::EmptyField("request_id"))?,
        asset_id: row.try_get("", "asset_id").map_err(|_| DomainError::EmptyField("asset_id"))?,
        asset_title: row.try_get("", "asset_title").map_err(|_| DomainError::EmptyField("asset_title"))?,
        asset_location: row
            .try_get("", "asset_location")
            .map_err(|_| DomainError::EmptyField("asset_location"))?,
        description: row
            .try_get("", "description")
            .map_err(|_| DomainError::EmptyField("description"))?,
        priority: row.try_get("", "priority").map_err(|_| DomainError::EmptyField("priority"))?,
        status: row.try_get("", "status").map_err(|_| DomainError::EmptyField("status"))?,
        assignee: row.try_get("", "assignee").ok(),
        open_escalation_count: row
            .try_get::<i32>("", "open_escalation_count")
            .map_err(|_| DomainError::EmptyField("open_escalation_count"))? as usize,
        work_order_count: row
            .try_get::<i32>("", "work_order_count")
            .map_err(|_| DomainError::EmptyField("work_order_count"))? as usize,
        overdue: row.try_get("", "overdue").map_err(|_| DomainError::EmptyField("overdue"))?,
        sla_deadline_epoch_sec: row
            .try_get::<i64>("", "sla_deadline_epoch_sec")
            .map_err(|_| DomainError::EmptyField("sla_deadline_epoch_sec"))? as u64,
        created_at_epoch_sec: row
            .try_get::<i64>("", "created_at_epoch_sec")
            .map_err(|_| DomainError::EmptyField("created_at_epoch_sec"))? as u64,
        owner_user_id: row
            .try_get("", "owner_user_id")
            .map_err(|_| DomainError::EmptyField("owner_user_id"))?,
    })
}

fn row_to_request_detail(row: sea_orm::QueryResult) -> Result<ReadRequestDetail, DomainError> {
    Ok(ReadRequestDetail {
        request_id: row.try_get("", "request_id").map_err(|_| DomainError::EmptyField("request_id"))?,
        asset_id: row.try_get("", "asset_id").map_err(|_| DomainError::EmptyField("asset_id"))?,
        asset_title: row.try_get("", "asset_title").map_err(|_| DomainError::EmptyField("asset_title"))?,
        asset_location: row
            .try_get("", "asset_location")
            .map_err(|_| DomainError::EmptyField("asset_location"))?,
        description: row
            .try_get("", "description")
            .map_err(|_| DomainError::EmptyField("description"))?,
        priority: row.try_get("", "priority").map_err(|_| DomainError::EmptyField("priority"))?,
        status: row.try_get("", "status").map_err(|_| DomainError::EmptyField("status"))?,
        assignee: row.try_get("", "assignee").ok(),
        work_order_status: row.try_get("", "work_order_status").ok(),
        open_escalation_count: row
            .try_get::<i32>("", "open_escalation_count")
            .map_err(|_| DomainError::EmptyField("open_escalation_count"))? as usize,
        latest_escalation_reason: row.try_get("", "latest_escalation_reason").ok(),
        overdue: row.try_get("", "overdue").map_err(|_| DomainError::EmptyField("overdue"))?,
        sla_deadline_epoch_sec: row
            .try_get::<i64>("", "sla_deadline_epoch_sec")
            .map_err(|_| DomainError::EmptyField("sla_deadline_epoch_sec"))? as u64,
        created_at_epoch_sec: row
            .try_get::<i64>("", "created_at_epoch_sec")
            .map_err(|_| DomainError::EmptyField("created_at_epoch_sec"))? as u64,
        owner_user_id: row
            .try_get("", "owner_user_id")
            .map_err(|_| DomainError::EmptyField("owner_user_id"))?,
    })
}

fn request_to_list_item(
    now_epoch: u64,
    assets: &HashMap<String, Asset>,
    work_orders: &[WorkOrder],
    escalations: &[Escalation],
    request: ServiceRequest,
) -> Result<ReadRequestListItem, DomainError> {
    let asset = assets
        .get(&request.asset_id)
        .ok_or(DomainError::NotFound("asset"))?;
    let related_work_orders = work_orders
        .iter()
        .filter(|item| item.request_id == request.id)
        .collect::<Vec<_>>();
    let related_escalations = escalations
        .iter()
        .filter(|item| item.request_id == request.id && item.state == EscalationState::Open)
        .collect::<Vec<_>>();
    let assignee = related_work_orders.iter().rev().find_map(|item| item.assignee.clone());
    let sla_deadline_epoch_sec = request.created_at_epoch_sec + (request.sla_minutes as u64) * 60;
    let overdue = !request.is_terminal() && now_epoch > sla_deadline_epoch_sec;
    Ok(ReadRequestListItem {
        request_id: request.id,
        asset_id: request.asset_id,
        asset_title: asset.title.clone(),
        asset_location: asset.location.clone(),
        description: request.description,
        priority: format!("{:?}", request.priority),
        status: format!("{:?}", request.status),
        assignee,
        open_escalation_count: related_escalations.len(),
        work_order_count: related_work_orders.len(),
        overdue,
        sla_deadline_epoch_sec,
        created_at_epoch_sec: request.created_at_epoch_sec,
        owner_user_id: request.owner_user_id,
    })
}

fn build_detail(
    now_epoch: u64,
    asset: &Asset,
    request: ServiceRequest,
    work_orders: &[WorkOrder],
    escalations: &[Escalation],
) -> ReadRequestDetail {
    let latest_work_order = work_orders.last();
    let open_count = escalations
        .iter()
        .filter(|item| item.state == EscalationState::Open)
        .count();
    let latest_reason = escalations.last().map(|item| item.reason.clone());
    let sla_deadline_epoch_sec = request.created_at_epoch_sec + (request.sla_minutes as u64) * 60;
    let overdue = !request.is_terminal() && now_epoch > sla_deadline_epoch_sec;
    ReadRequestDetail {
        request_id: request.id,
        asset_id: request.asset_id,
        asset_title: asset.title.clone(),
        asset_location: asset.location.clone(),
        description: request.description,
        priority: format!("{:?}", request.priority),
        status: format!("{:?}", request.status),
        assignee: latest_work_order.and_then(|item| item.assignee.clone()),
        work_order_status: latest_work_order.map(|item| format!("{:?}", item.status)),
        open_escalation_count: open_count,
        latest_escalation_reason: latest_reason,
        overdue,
        sla_deadline_epoch_sec,
        created_at_epoch_sec: request.created_at_epoch_sec,
        owner_user_id: request.owner_user_id,
    }
}

fn apply_filter(items: &mut Vec<ReadRequestListItem>, filter: RequestListFilter) {
    if let Some(status) = filter.status {
        items.retain(|item| item.status.eq_ignore_ascii_case(&status));
    }
    if let Some(priority) = filter.priority {
        items.retain(|item| item.priority.eq_ignore_ascii_case(&priority));
    }
    if filter.overdue_only {
        items.retain(|item| item.overdue);
    }
    *items = items
        .iter()
        .skip(filter.offset)
        .take(filter.limit.min(500))
        .cloned()
        .collect();
}

fn compute_sla_summary(items: &[ReadRequestListItem]) -> SlaComplianceSummary {
    let open = items
        .iter()
        .filter(|item| !is_terminal_status(&item.status))
        .collect::<Vec<_>>();
    let overdue = open.iter().filter(|item| item.overdue).count();
    let total = open.len();
    let compliant = total.saturating_sub(overdue);
    SlaComplianceSummary {
        total_open_requests: total,
        overdue_open_requests: overdue,
        compliant_open_requests: compliant,
        compliance_percent: if total == 0 {
            100.0
        } else {
            (compliant as f64 / total as f64) * 100.0
        },
    }
}

fn compute_sla_by_priority(items: &[ReadRequestListItem]) -> Vec<SlaComplianceByPriorityItem> {
    let mut grouped = HashMap::<String, Vec<&ReadRequestListItem>>::new();
    for item in items.iter().filter(|item| !is_terminal_status(&item.status)) {
        grouped.entry(item.priority.clone()).or_default().push(item);
    }
    let mut out = grouped
        .into_iter()
        .map(|(priority, rows)| {
            let total = rows.len();
            let overdue = rows.iter().filter(|item| item.overdue).count();
            let compliant = total.saturating_sub(overdue);
            SlaComplianceByPriorityItem {
                priority,
                total_open_requests: total,
                overdue_open_requests: overdue,
                compliant_open_requests: compliant,
                compliance_percent: if total == 0 {
                    100.0
                } else {
                    (compliant as f64 / total as f64) * 100.0
                },
            }
        })
        .collect::<Vec<_>>();
    out.sort_by(|a, b| a.priority.cmp(&b.priority));
    out
}

fn is_terminal_status(status: &str) -> bool {
    matches!(status, "Resolved" | "Closed")
}

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[allow(dead_code)]
fn _status_name(status: WorkOrderStatus) -> &'static str {
    match status {
        WorkOrderStatus::Created => "Created",
        WorkOrderStatus::Assigned => "Assigned",
        WorkOrderStatus::InProgress => "InProgress",
        WorkOrderStatus::Completed => "Completed",
        WorkOrderStatus::Cancelled => "Cancelled",
    }
}

#[allow(dead_code)]
fn _request_status_name(status: RequestStatus) -> &'static str {
    match status {
        RequestStatus::New => "New",
        RequestStatus::Planned => "Planned",
        RequestStatus::InProgress => "InProgress",
        RequestStatus::Resolved => "Resolved",
        RequestStatus::Closed => "Closed",
        RequestStatus::Escalated => "Escalated",
    }
}

#[allow(dead_code)]
fn _technician_name(_item: Technician) {}
