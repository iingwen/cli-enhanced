use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use crate::{
    error::{Error, Result},
    resources::user::Username,
};

static FULL_NAME_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[A-Za-z0-9-_]{1,256}/[A-Za-z0-9-_]{1,256}$").unwrap());

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Bucket {
    pub id: Id,
    pub name: Name,
    pub owner: Username,
    pub created_at: DateTime<Utc>,
}

impl Bucket {
    pub fn full_name(&self) -> FullName {
        FullName(format!("{}/{}", self.owner.0, self.name.0))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ModelFamily(pub String);

// TODO(mcobzarenco)[3963]: Make `Identifier` into a trait (ensure it still implements
// `FromStr` so we can take T: Identifier as a clap command line argument).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Identifier {
    Id(Id),
    FullName(FullName),
}

impl From<FullName> for Identifier {
    fn from(full_name: FullName) -> Self {
        Identifier::FullName(full_name)
    }
}

impl From<Id> for Identifier {
    fn from(id: Id) -> Self {
 