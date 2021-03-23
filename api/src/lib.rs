#![deny(clippy::all)]
mod error;
pub mod resources;
pub mod retry;

use chrono::{DateTime, Utc};
use http::Method;
use log::debug;
use once_cell::sync::Lazy;
use reqwest::{
    blocking::{multipart::Form, Client as HttpClient, Response as HttpResponse},
    header::{self, HeaderMap, HeaderValue},
    IntoUrl, Proxy, Result as ReqwestResult,
};
use resources::{
    bucket_statistics::GetBucketStatisticsResponse,
    comment::CommentTimestampFilter,
    dataset::{
        QueryRequestParams, QueryResponse,
        StatisticsRequestParams as DatasetStatisticsRequestParams, SummaryRequestParams,
        SummaryResponse,
    },
    documents::{Document, SyncRawEmailsRequest, SyncRawEmailsResponse},
    integration::{
        GetIntegrationResponse, GetIntegrationsResponse, Integration, NewIntegration,
        PostIntegrationRequest, PostIntegrationResponse, PutIntegrationRequest,
        PutIntegrationResponse,
    },
    project::ForceDeleteProject,
    quota::{GetQuotasResponse, Quota},
    source::StatisticsRequestParams as SourceStatisticsRequestParams,
    stream::{GetStreamResponse, NewStream, PutStreamRequest, PutStreamResponse},
    validation::{
        LabelValidation, LabelValidationRequest, LabelValidationResponse, ValidationResponse,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{cell::Cell, fmt::Display, path::Path};
use url::Url;

use crate::resources::{
    audit::{AuditQueryFilter, AuditQueryRequest, AuditQueryResponse},
    bucket::{
        CreateRequest as CreateBucketRequest, CreateResponse as CreateBucketResponse,
        GetAvailableResponse as GetAvailableBucketsResponse, GetResponse as GetBucketResponse,
    },
    bucket_statistics::Statistics as BucketStatistics,
    comment::{
        GetAnnotationsResponse, GetCommentResponse, GetLabellingsAfter, GetPredictionsResponse,
        GetRecentRequest, PutCommentsRequest, PutCommentsResponse, RecentCommentsPage,
        SyncCommentsRequest, UpdateAnnotationsRequest,
    },
    dataset::{
        CreateRequest as CreateDatasetRequest, CreateResponse as CreateDatasetResponse,
        GetAvailableResponse as GetAvailableDatasetsResponse, GetResponse as GetDatasetResponse,
        UpdateRequest as UpdateDatasetRequest, UpdateResponse as UpdateDatasetResponse,
    },
    email::{PutEmailsRequest, PutEmailsResponse},
    project::{
        CreateProjectRequest, CreateProjectResponse, GetProjectResponse, GetProjectsResponse,
        UpdateProjectRequest, UpdateProjectResponse,
    },
    quota::{CreateQuota, TenantQuotaKind},
    source::{
        CreateRequest as CreateSourceRequest, CreateResponse as CreateSourceResponse,
        GetAvailableResponse as GetAvailableSourcesResponse, GetResponse as GetSourceResponse,
        UpdateRequest as UpdateSourceRequest, UpdateResponse as UpdateSourceResponse,
    },
    statistics::GetResponse as GetStatisticsResponse,
    stream::{
        AdvanceRequest as StreamAdvanceRequest, FetchRequest as StreamFetchRequest,
        GetStreamsResponse, ResetRequest as StreamResetRequest,
        TagExceptionsRequest as TagStreamExceptionsRequest,
    },
    tenant_id::TenantId,
    user::GetResponse as GetUserResponse,
    user::{
        CreateRequest as CreateUserRequest, CreateResponse as CreateUserResponse,
        GetAvailableResponse as GetAvailableUsersResponse,
        GetCurrentResponse as GetCurrentUserResponse, PostUserRequest, PostUserResponse,
        WelcomeEmailResponse,
    },
    EmptySuccess, Response,
};

use crate::retry::{Retrier, RetryConfig};

pub use crate::{
    error::{Error, Result},
    resources::{
        bucket::{
            Bucket, BucketType, FullName as BucketFullName, Id as BucketId,
            Identifier as BucketIdentifier, Name as BucketName, NewBucket,
        },
        comment::{
            AnnotatedComment, Comment, CommentFilter, CommentsIterPage, Continuation,
            EitherLabelling, Entities, Entity, HasAnnotations, Id as CommentId, Label, Labelling,
            Message, MessageBody, MessageSignature, MessageSubject, NewAnnotatedComment,
            NewComment, NewEntities, NewLabelling, NewMoonForm, PredictedLabel, Prediction,
            PropertyMap, PropertyValue, Sentiment, SyncCommentsResponse, Uid as CommentUid,
        },
        dataset::{
            Dataset, FullName as DatasetFullName, Id as DatasetId, Identifier as DatasetIdentifier,
            ModelVersion, Name as DatasetName, NewDataset, UpdateDataset,
        },
        email::{
            Continuation as EmailContinuation, EmailsIterPage, Id as EmailId, Mailbox, MimeContent,
            NewEmail,
        },
        entity_def::{EntityDef, Id as EntityDefId, Name as EntityName, NewEntityDef},
        integration::FullName as IntegrationFullName,
        label_def::{
            LabelDef, LabelDefPretrained, MoonFormFieldDef, Name as LabelName, NewLabelDef,
            NewLabelDefPretrained, PretrainedId as LabelDefPretrainedId,
        },
        label_group::{
            LabelGroup, Name as LabelGroupName, NewLabelGroup, DEFAULT_LABEL_GROUP_NAME,
        },
        project::{NewProject, Project, ProjectName, UpdateProject},
        source::{
            FullName as SourceFullName, Id as SourceId, Identifier as SourceIdentifier,
            Name as SourceName, NewSource, Source, SourceKind, TransformTag, UpdateSource,
        },
        statistics::Statistics as CommentStatistics,
        stream::{
            Batch as StreamBatch, FullName as StreamFullName, SequenceId as StreamSequenceId,
            Stream, StreamException, StreamExceptionMetadata,
        },
        user::{
            Email, GlobalPermission, Id as UserId, Identifier as UserIdentifier,
            ModifiedPermissions, NewUser, ProjectPermission, UpdateUser, User, Username,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token(pub String);

pub struct Config {
    pub endpoint: Url,
    pub token: Token,
    pub accept_invalid_certificates: bool,
    pub proxy: Option<Url>,
    /// Retry settings to use, if any. This will apply to all requests except for POST requests
    /// which are not idempotent (as they cannot be naively retried).
    pub retry_config: Option<RetryConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            endpoint: DEFAULT_ENDPOINT.clone(),
            token: Token("".to_owned()),
            accept_invalid_certificates: false,
            proxy: None,
            retry_config: None,
        }
    }
}

#[derive(Debug)]
pub struct Client {
    endpoints: Endpoints,
    http_client: HttpClient,
    headers: HeaderMap,
    retrier: Option<Retrier>,
}

#[derive(Serialize)]
pub struct GetLabellingsInBulk<'a> {
    pub source_id: &'a SourceId,
    pub return_predictions: &'a bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: &'a Option<GetLabellingsAfter>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: &'a Option<usize>,
}

#[derive(Serialize)]
pub struct GetCommentsIterPageQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_timestamp: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_timestamp: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<&'a Continuation>,
    pub limit: usize,
    pub include_markup: bool,
}

#[derive(Serialize)]
pub struct GetEmailsIterPageQuery<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<&'a EmailContinuation>,
    pub limit: usize,
}

#[derive(Serialize)]
pub struct GetCommentQuery {
    pub include_markup: bool,
}

impl Client {
    /// Create a new API client.
    pub fn new(config: Config) -> Result<Client> {
        let http_client = build_http_client(&config)?;
        let headers = build_headers(&config)?;
        let endpoints = Endpoints::new(config.endpoint)?;
        let retrier = config.retry_config.map(Retrier::new);
        Ok(Client {
            endpoints,
            http_client,
            headers,
            retrier,
        })
    }

    /// Get the base url for the client
    pub fn base_url(&self) -> &Url {
        &self.endpoints.base
    }

    /// List all visible sources.
    pub fn get_sources(&self) -> R