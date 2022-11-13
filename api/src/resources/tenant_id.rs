use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReinferTenantId(String);

#[derive(Serialize, Deserialize, Debug, Clo