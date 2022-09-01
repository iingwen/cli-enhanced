
use crate::{CommentId, NewComment, PropertyMap, TransformTag};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

use super::email::AttachmentMetadata;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub raw_email: RawEmail,
    pub user_properties: PropertyMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<CommentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawEmail {
    pub body: RawEmailBody,
    pub headers: RawEmailHeaders,
    pub attachments: Vec<AttachmentMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawEmailBody {
    Plain(String),
    Html(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RawEmailHeaders {
    Raw(String),