use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use crate::{
    error::{Error, Result},
    resources::bucket::Id as BucketId,
    resources::user::Username,
    CommentFilter,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct TransformTag(pub String);

impl FromStr for TransformTag {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(Self(string.to_owned()))
    }
}

#[derive(Debug, Clone, S