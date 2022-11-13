use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReinferTenantId(String);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiPathTenantId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TenantId {
    Reinfer(ReinferTenantId),
    UiPath(UiPathTenantId),
}

impl Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TenantId::Reinfer(ReinferTenantId(tenant_id))
                | TenantId::UiPath(UiPathTenantId(tenant_id)) => tenant_id,
            }
     