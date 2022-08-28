
use crate::{
    error::{Error, Result},
    resources::entity_def::Name as EntityName,
    resources::label_def::Name as LabelName,
    resources::label_group::Name as LabelGroupName,
    resources::label_group::DEFAULT_LABEL_GROUP_NAME,
    SourceId,
};
use chrono::{DateTime, Utc};
use ordered_float::NotNan;
use serde::{
    de::{Deserializer, Error as SerdeError, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    fmt::{Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
    path::PathBuf,
    result::Result as StdResult,
    str::FromStr,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(pub String);

impl FromStr for Id {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(Self(string.to_owned()))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Uid(pub String);

impl FromStr for Uid {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        Ok(Self(string.to_owned()))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ThreadId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommentTimestampFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewedFilterEnum {
    OnlyReviewed,
    OnlyUnreviewed,
}

type UserPropertyName = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPropertiesFilter(pub HashMap<UserPropertyName, PropertyFilter>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyFilter {
    #[serde(skip_serializing_if = "<[_]>::is_empty", default)]
    pub one_of: Vec<PropertyValue>,
    #[serde(skip_serializing_if = "<[_]>::is_empty", default)]
    pub not_one_of: Vec<PropertyValue>,
    #[serde(skip_serializing_if = "<[_]>::is_empty", default)]
    pub domain_not_one_of: Vec<PropertyValue>,
}

impl PropertyFilter {
    pub fn new(
        one_of: Vec<PropertyValue>,
        not_one_of: Vec<PropertyValue>,
        domain_not_one_of: Vec<PropertyValue>,
    ) -> Self {
        Self {
            one_of,
            not_one_of,
            domain_not_one_of,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommentFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed: Option<ReviewedFilterEnum>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<CommentTimestampFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_properties: Option<UserPropertiesFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub sources: Vec<SourceId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<MessagesFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MessagesFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<PropertyFilter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<PropertyFilter>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GetRecentRequest<'a> {
    pub limit: usize,
    pub filter: &'a CommentFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<&'a Continuation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Continuation(pub String);

#[derive(Debug, Clone, Deserialize)]
pub struct RecentCommentsPage {
    pub results: Vec<AnnotatedComment>,
    pub continuation: Option<Continuation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct GetLabellingsAfter(pub String);

#[derive(Debug, Clone, Deserialize)]
pub struct GetAnnotationsResponse {
    pub results: Vec<AnnotatedComment>,
    #[serde(default)]
    pub after: Option<GetLabellingsAfter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetPredictionsResponse {
    pub predictions: Vec<Prediction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateAnnotationsRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelling: Option<&'a [NewLabelling]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<&'a NewEntities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moon_forms: Option<&'a [NewMoonForm]>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentsIterPage {
    pub comments: Vec<Comment>,
    pub continuation: Option<Continuation>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PutCommentsRequest<'request> {
    pub comments: &'request [NewComment],
}

#[derive(Debug, Clone, Deserialize)]
pub struct PutCommentsResponse;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SyncCommentsRequest<'request> {
    pub comments: &'request [NewComment],
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncCommentsResponse {
    pub new: usize,
    pub updated: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GetCommentResponse {
    pub comment: Comment,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Comment {
    pub id: Id,
    pub uid: Uid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<ThreadId>,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "PropertyMap::is_empty", default)]
    pub user_properties: PropertyMap,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<AttachmentMetadata>,
    pub created_at: DateTime<Utc>,

    #[serde(default)]
    pub has_annotations: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewComment {
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<ThreadId>,
    pub timestamp: DateTime<Utc>,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "PropertyMap::is_empty", default)]
    pub user_properties: PropertyMap,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attachments: Vec<AttachmentMetadata>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Message {
    pub body: MessageBody,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<MessageSubject>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<MessageSignature>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MessageBody {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_markup: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from_markup: Option<JsonValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MessageSubject {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MessageSignature {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_markup: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translated_from_markup: Option<JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Eq)]
pub enum Sentiment {
    #[serde(rename = "positive")]
    Positive,

    #[serde(rename = "negative")]
    Negative,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AttachmentMetadata {
    pub name: String,
    pub size: u64,
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub struct PropertyMap(HashMap<String, PropertyValue>);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Number(NotNan<f64>),
}
