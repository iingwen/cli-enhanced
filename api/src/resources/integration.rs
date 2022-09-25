
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{Email, ProjectName};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewIntegration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    pub configuration: Configuration,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Integration {
    pub id: Id,
    pub owner: ProjectName,
    pub name: FullName,
    pub title: Title,
    #[serde(rename = "type")]
    pub integration_type: IntegrationType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled: bool,
    pub disabled_reason: Option<DisabledReason>,
    pub configuration: Configuration,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

impl FromStr for FullName {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.split('/').count() == 2 {
            Ok(FullName(string.into()))
        } else {
            Err(Error::BadIntegrationIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Title(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationType {
    ExchangeOnline,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisabledReason {
    User,
    Quota,
    SyncError,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Configuration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<Connection>,
    pub mailboxes: Vec<Mailbox>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Connection {
    access: AccessType,
    application: ApplicationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    ews_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    build_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AccessType {
    UserAccess(UserAccessConfig),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]