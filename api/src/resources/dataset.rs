
use chrono::{DateTime, Utc};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    resources::{
        entity_def::{EntityDef, NewEntityDef},
        label_def::{LabelDef, NewLabelDef},
        label_group::{LabelGroup, NewLabelGroup},
        source::Id as SourceId,
        user::Username,
    },
    AnnotatedComment, CommentFilter, Continuation,
};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Dataset {
    pub id: Id,
    pub name: Name,
    pub owner: Username,
    pub title: String,
    pub description: String,
    #[serde(rename = "created")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "last_modified")]
    pub updated_at: DateTime<Utc>,
    pub model_family: ModelFamily,
    pub source_ids: Vec<SourceId>,
    pub has_sentiment: bool,
    pub entity_defs: Vec<EntityDef>,
    pub label_defs: Vec<LabelDef>,
    pub label_groups: Vec<LabelGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DatasetStats {
    pub num_reviewed: NotNan<f64>,
    pub total_verbatims: NotNan<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct DatasetAndStats {
    pub dataset: Dataset,
    pub stats: DatasetStats,
}

impl Dataset {
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
        if string.split('/').count() == 2 {
            Ok(FullName(string.into()))
        } else {
            Err(Error::BadDatasetIdentifier {
                identifier: string.into(),
            })
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeResolution {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Attribute {
    Labels,
    AttachmentPropertyTypes,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttributeFilterEnum {
    StringAnyOf { any_of: Vec<String> },
}

#[derive(Debug, Clone, Serialize)]
pub struct AttributeFilter {
    pub attribute: Attribute,
    pub filter: AttributeFilterEnum,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StatisticsRequestParams {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub attribute_filters: Vec<AttributeFilter>,

    pub comment_filter: CommentFilter,

    pub label_property_timeseries: bool,

    pub label_timeseries: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_resolution: Option<TimeResolution>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrderEnum {
    ByLabel { label: String },
    Recent,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryRequestParams {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub attribute_filters: Vec<AttributeFilter>,

    pub filter: CommentFilter,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryRequestParams {
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub attribute_filters: Vec<AttributeFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<Continuation>,

    pub filter: CommentFilter,

    pub limit: usize,

    pub order: OrderEnum,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPropertySummary {
    pub full_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPropertySummaryList {
    pub string: Vec<UserPropertySummary>,
    pub number: Vec<UserPropertySummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Summary {
    pub user_properties: UserPropertySummaryList,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SummaryResponse {
    pub summary: Summary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryResponse {
    pub continuation: Option<Continuation>,
    pub results: Vec<AnnotatedComment>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ModelFamily(pub String);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ModelVersion(pub u32);

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
        Identifier::Id(id)
    }
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Identifier::Id(Id(string.into())))
        } else {
            FullName::from_str(string).map(Identifier::FullName)
        }
    }
}