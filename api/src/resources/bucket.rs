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

#[derive(Debug, C