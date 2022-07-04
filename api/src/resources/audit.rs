
use chrono::{DateTime, Utc};

use crate::{Continuation, DatasetId, DatasetName, Email, ProjectName, UserId, Username};

use super::{comment::CommentTimestampFilter, project::Id as ProjectId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditQueryFilter {
    pub timestamp: CommentTimestampFilter,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AuditEventId(pub String);

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AuditEventType(pub String);

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AuditTenantName(pub String);

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AuditTenantId(pub String);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditQueryRequest {
    pub filter: AuditQueryFilter,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub continuation: Option<Continuation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditEvent {
    actor_user_id: UserId,
    actor_tenant_id: AuditTenantId,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    dataset_ids: Vec<DatasetId>,
    event_id: AuditEventId,
    event_type: AuditEventType,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    project_ids: Vec<ProjectId>,
    tenant_ids: Vec<AuditTenantId>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrintableAuditEvent {
    pub actor_email: Email,
    pub actor_tenant_name: AuditTenantName,
    pub event_type: AuditEventType,
    pub dataset_names: Vec<DatasetName>,
    pub event_id: AuditEventId,
    pub project_names: Vec<ProjectName>,
    pub tenant_names: Vec<AuditTenantName>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuditDataset {
    id: DatasetId,
    name: DatasetName,
    project_id: ProjectId,
    title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuditProject {
    id: ProjectId,
    name: ProjectName,
    tenant_id: AuditTenantId,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuditTenant {
    id: AuditTenantId,
    name: AuditTenantName,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AuditUser {
    display_name: Username,
    email: Email,
    id: UserId,
    tenant_id: AuditTenantId,
    username: Username,
}