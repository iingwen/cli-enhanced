
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

impl Deref for PropertyMap {
    type Target = HashMap<String, PropertyValue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PropertyMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PropertyMap {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        PropertyMap(HashMap::with_capacity(capacity))
    }

    #[inline]
    pub fn insert_number(&mut self, key: String, value: NotNan<f64>) {
        self.0.insert(key, PropertyValue::Number(value));
    }

    #[inline]
    pub fn insert_string(&mut self, key: String, value: String) {
        self.0.insert(key, PropertyValue::String(value));
    }

    // Provided despite deref, for `skip_serializing_if`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

const STRING_PROPERTY_PREFIX: &str = "string:";
const NUMBER_PROPERTY_PREFIX: &str = "number:";

impl Serialize for PropertyMap {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        let mut state = serializer.serialize_map(Some(self.len()))?;
        if self.0.is_empty() {
            return state.end();
        }

        let mut full_name = String::with_capacity(32);
        for (key, value) in &self.0 {
            full_name.clear();
            match value {
                PropertyValue::String(value) => {
                    if !value.trim().is_empty() {
                        full_name.push_str(STRING_PROPERTY_PREFIX);
                        full_name.push_str(key);
                        state.serialize_entry(&full_name, &value)?;
                    }
                }
                PropertyValue::Number(value) => {
                    full_name.push_str(NUMBER_PROPERTY_PREFIX);
                    full_name.push_str(key);
                    state.serialize_entry(&full_name, &value)?;
                }
            }
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for PropertyMap {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        deserializer.deserialize_any(PropertyMapVisitor)
    }
}

struct PropertyMapVisitor;
impl<'de> Visitor<'de> for PropertyMapVisitor {
    type Value = PropertyMap;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "a user property map")
    }

    #[inline]
    fn visit_unit<E>(self) -> StdResult<PropertyMap, E> {
        Ok(PropertyMap::new())
    }

    fn visit_map<M>(self, mut access: M) -> StdResult<PropertyMap, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut values = PropertyMap::with_capacity(access.size_hint().unwrap_or(0));

        while let Some(mut key) = access.next_key()? {
            if strip_prefix(&mut key, STRING_PROPERTY_PREFIX) {
                values.insert(key, PropertyValue::String(access.next_value()?));
            } else if strip_prefix(&mut key, NUMBER_PROPERTY_PREFIX) {
                values.insert(key, PropertyValue::Number(access.next_value()?));
            } else {
                return Err(M::Error::custom(format!(
                    "user property full name `{key}` has invalid \
                     type prefix"
                )));
            }
        }

        Ok(values)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnnotatedComment {
    pub comment: Comment,
    #[serde(skip_serializing_if = "should_skip_serializing_labelling")]
    pub labelling: Option<Vec<Labelling>>,
    #[serde(skip_serializing_if = "should_skip_serializing_entities")]
    pub entities: Option<Entities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_properties: Option<ThreadProperties>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub moon_forms: Option<Vec<MoonForm>>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub label_properties: Option<Vec<LabelProperty>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Prediction {
    pub uid: Uid,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec")]
    pub labels: Option<Vec<AutoThresholdLabel>>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec")]
    pub entities: Option<Vec<Entity>>,
}

pub fn get_default_labelling_group(labelling: &Option<Vec<Labelling>>) -> Option<&Labelling> {
    labelling
        .iter()
        .flatten()
        .find(|&labelling_group| labelling_group.is_default_group())
}

impl Labelling {
    pub fn is_default_group(&self) -> bool {
        self.group == *DEFAULT_LABEL_GROUP_NAME
    }
}

impl NewLabelling {
    pub fn is_default_group(&self) -> bool {
        self.group == *DEFAULT_LABEL_GROUP_NAME
    }
}

impl HasAnnotations for AnnotatedComment {
    fn has_annotations(&self) -> bool {
        let has_labels = self.labelling.iter().flatten().any(|labelling_group| {
            !labelling_group.assigned.is_empty() || !labelling_group.dismissed.is_empty()
        });
        let has_entities = self
            .entities
            .as_ref()
            .map(|entities| !entities.assigned.is_empty() || !entities.dismissed.is_empty())
            .unwrap_or(false);
        has_labels || has_entities || self.moon_forms.has_annotations()
    }
}

impl AnnotatedComment {
    pub fn without_predictions(mut self) -> Self {
        self.labelling = self.labelling.and_then(|mut labelling| {
            if labelling.iter().all(|labelling_group| {
                labelling_group.assigned.is_empty() && labelling_group.dismissed.is_empty()
            }) {
                None
            } else {
                for comment_labelling in &mut labelling {
                    comment_labelling.predicted = None;
                }
                Some(labelling)
            }
        });
        self.entities = self.entities.and_then(|mut entities| {
            if entities.assigned.is_empty() && entities.dismissed.is_empty() {
                None
            } else {
                entities.predicted = None;
                Some(entities)
            }
        });
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ThreadProperties {
    duration: Option<NotNan<f64>>,
    response_time: Option<NotNan<f64>>,
    num_messages: u64,
    thread_position: Option<u64>,
    first_sender: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum EitherLabelling {
    Labelling(Vec<NewLabelling>),
    LegacyLabelling(NewLegacyLabelling),
}

impl EitherLabelling {
    fn into_new_labellings(self) -> Vec<NewLabelling> {
        match self {
            EitherLabelling::Labelling(new_labelling_vec) => new_labelling_vec,
            EitherLabelling::LegacyLabelling(new_legacy_labelling) => {
                vec![NewLabelling {
                    group: DEFAULT_LABEL_GROUP_NAME.clone(),
                    assigned: new_legacy_labelling.assigned,
                    dismissed: new_legacy_labelling.dismissed,
                }]
            }
        }
    }
}

impl From<EitherLabelling> for Vec<NewLabelling> {
    fn from(either_labelling: EitherLabelling) -> Vec<NewLabelling> {
        either_labelling.into_new_labellings()
    }
}

impl HasAnnotations for EitherLabelling {
    fn has_annotations(&self) -> bool {
        match self {
            EitherLabelling::Labelling(new_labelling) => {
                new_labelling.iter().any(|labelling_group| {
                    labelling_group.assigned.is_some() || labelling_group.dismissed.is_some()
                })
            }
            EitherLabelling::LegacyLabelling(new_legacy_labelling) => {
                new_legacy_labelling.assigned.is_some() || new_legacy_labelling.dismissed.is_some()
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewAnnotatedComment {
    pub comment: NewComment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelling: Option<EitherLabelling>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entities: Option<NewEntities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "should_skip_serializing_optional_vec", default)]
    pub moon_forms: Option<Vec<NewMoonForm>>,
}

impl<T> HasAnnotations for Option<T>
where
    T: HasAnnotations,
{
    fn has_annotations(&self) -> bool {
        self.as_ref().map_or(false, HasAnnotations::has_annotations)
    }
}

impl HasAnnotations for Vec<MoonForm> {
    fn has_annotations(&self) -> bool {
        self.iter().any(|form| !form.assigned.is_empty())
    }
}

impl HasAnnotations for Vec<NewMoonForm> {
    fn has_annotations(&self) -> bool {
        self.iter().any(|form| !form.assigned.is_empty())
    }
}
