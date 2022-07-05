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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq