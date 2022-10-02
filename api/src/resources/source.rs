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

#[derive(Debug, Clone, Serialize, Default)]
pub struct StatisticsRequestParams {
    pub comment_filter: CommentFilter,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Source {
    pub id: Id,
    pub owner: Username,
    pub name: Name,
    pub title: String,
    pub description: String,
    pub language: String,
    pub should_translate: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bucket_id: Option<BucketId>,

    #[serde(rename = "_kind")]
    pub kind: SourceKind,
    #[serde(default, rename = "email_transform_tag")]
    pub transform_tag: Option<TransformTag>,
}

impl Source {
    pub fn full_name(&self) -> FullName {
        FullName(format!("{}/{}", self.owner.0, self.name.0))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FullName(pub String);

impl FromStr for FullName {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.split('/').