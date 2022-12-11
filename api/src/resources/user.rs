use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};

use super::project::ProjectName;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

impl FromStr for Id {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Id(string.into()))
        } else {
            Err(Error::BadUserIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Username(pub String);

impl FromStr for Username {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            Ok(Username(string.into()))
        } else {
            Err(Error::BadUserIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Email(pub String);

impl FromStr for Email {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(Email(string.into()))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Identifier {
    Id(Id),
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Identifier::Id(Id(string.into())))
        } else {
            Err(Error::BadUserIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct User {
    pub id: Id,
    pub username: Username,
    pub email: Email,
    #[serde(rename = "created")]
    pub created_at: DateTime<Utc>,
    pub global_permissions: HashSet<GlobalPermission>,
    #[serde(rename = "organisation_permissions")]
    pub project_permissions: HashMap<ProjectName, HashSet<ProjectPermission>>,
    pub sso_global_permissions: HashSet<GlobalPermission>,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NewUser<'r> {
    pub username: &'r Username,
    pub email: &'r Email,
    pub global_permissions: &'r [GlobalPermission],
    #[serde(rename = "organisation_permissions")]
    pub project_permissions: &'r HashMap<ProjectName, HashSet<ProjectPermission>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModifiedPermissions<'r> {
    #[serde(
        rename = "organisation_permissions",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub project_permissions: &'r HashMap<ProjectName, HashSet<ProjectPermission>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub global_permissions: Vec<&'r GlobalPermission>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CreateRequest<'request> {
    pub user: NewUser<'request>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CreateResponse {
    pub user: User,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetAvailableResponse {
    pub users: Vec<User>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetCurrentResponse {
    pub user: User,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct GetResponse {
    pub user: User,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct WelcomeEmailResponse {}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ProjectPermission {
    // TODO(jcalero)[RE-978] There is a bug with the im