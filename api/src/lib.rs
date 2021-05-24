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
    pub fn get_sources(&self) -> Result<Vec<Source>> {
        Ok(self
            .get::<_, GetAvailableSourcesResponse>(self.endpoints.sources.clone())?
            .sources)
    }

    /// Get a source by either id or name.
    pub fn get_user(&self, user: impl Into<UserIdentifier>) -> Result<User> {
        Ok(match user.into() {
            UserIdentifier::Id(user_id) => {
                self.get::<_, GetUserResponse>(self.endpoints.user_by_id(&user_id)?)?
                    .user
            }
        })
    }

    /// Get a source by either id or name.
    pub fn get_source(&self, source: impl Into<SourceIdentifier>) -> Result<Source> {
        Ok(match source.into() {
            SourceIdentifier::Id(source_id) => {
                self.get::<_, GetSourceResponse>(self.endpoints.source_by_id(&source_id)?)?
                    .source
            }
            SourceIdentifier::FullName(source_name) => {
                self.get::<_, GetSourceResponse>(self.endpoints.source_by_name(&source_name)?)?
                    .source
            }
        })
    }

    /// Create a new source.
    pub fn create_source(
        &self,
        source_name: &SourceFullName,
        options: NewSource<'_>,
    ) -> Result<Source> {
        Ok(self
            .put::<_, _, CreateSourceResponse>(
                self.endpoints.source_by_name(source_name)?,
                CreateSourceRequest { source: options },
            )?
            .source)
    }

    /// Update a source.
    pub fn update_source(
        &self,
        source_name: &SourceFullName,
        options: UpdateSource<'_>,
    ) -> Result<Source> {
        Ok(self
            .post::<_, _, UpdateSourceResponse>(
                self.endpoints.source_by_name(source_name)?,
                UpdateSourceRequest { source: options },
                Retry::Yes,
            )?
            .source)
    }

    /// Delete a source.
    pub fn delete_source(&self, source: impl Into<SourceIdentifier>) -> Result<()> {
        let source_id = match source.into() {
            SourceIdentifier::Id(source_id) => source_id,
            source @ SourceIdentifier::FullName(_) => self.get_source(source)?.id,
        };
        self.delete(self.endpoints.source_by_id(&source_id)?)
    }

    /// Set a quota
    pub fn create_quota(
        &self,
        target_tenant_id: &TenantId,
        tenant_quota_kind: TenantQuotaKind,
        options: CreateQuota,
    ) -> Result<()> {
        self.post(
            self.endpoints.quota(target_tenant_id, tenant_quota_kind)?,
            options,
            Retry::Yes,
        )
    }

    /// Get quotas for current tenant
    pub fn get_quotas(&self) -> Result<Vec<Quota>> {
        Ok(self
            .get::<_, GetQuotasResponse>(self.endpoints.quotas()?)?
            .quotas)
    }

    /// Delete a user.
    pub fn delete_user(&self, user: impl Into<UserIdentifier>) -> Result<()> {
        let UserIdentifier::Id(user_id) = user.into();
        self.delete(self.endpoints.user_by_id(&user_id)?)
    }

    /// Delete comments by id in a source.
    pub fn delete_comments(
        &self,
        source: impl Into<SourceIdentifier>,
        comments: &[CommentId],
    ) -> Result<()> {
        let source_full_name = match source.into() {
            source @ SourceIdentifier::Id(_) => self.get_source(source)?.full_name(),
            SourceIdentifier::FullName(source_full_name) => source_full_name,
        };
        self.delete_query(
            self.endpoints.comments_v1(&source_full_name)?,
            Some(&id_list_query(comments.iter().map(|uid| &uid.0))),
        )
    }

    /// Get a page of comments from a source.
    pub fn get_comments_iter_page(
        &self,
        source_name: &SourceFullName,
        continuation: Option<&ContinuationKind>,
        to_timestamp: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Result<CommentsIterPage> {
        // Comments are returned from the API in increasing order of their
        // `timestamp` field.
        let (from_timestamp, after) = match continuation {
            // If we have a timestamp, then this is a request for the first page of
            // a series of comments with timestamps starting from the given time.
            Some(ContinuationKind::Timestamp(from_timestamp)) => (Some(*from_timestamp), None),
            // If we have a continuation, then this is a request for page n+1 of
            // a series of comments, where the continuation came from page n.
            Some(ContinuationKind::Continuation(after)) => (None, Some(after)),
            // Otherwise, this is a request for the first page of a series of comments
            // with timestamps starting from the beginning of time.
            None => (None, None),
        };
        let query_params = GetCommentsIterPageQuery {
            from_timestamp,
            to_timestamp,
            after,
            limit,
            include_markup: true,
        };
        self.get_query(self.endpoints.comments(source_name)?, Some(&query_params))
    }

    /// Iterate through all comments for a given dataset query.
    pub fn get_dataset_query_iter<'a>(
        &'a self,
        dataset_name: &'a DatasetFullName,
        params: &'a mut QueryRequestParams,
    ) -> DatasetQueryIter<'a> {
        DatasetQueryIter::new(self, dataset_name, params)
    }

    /// Iterate through all comments in a source.
    pub fn get_comments_iter<'a>(
        &'a self,
        source_name: &'a SourceFullName,
        page_size: Option<usize>,
        timerange: CommentsIterTimerange,
    ) -> CommentsIter<'a> {
        CommentsIter::new(self, source_name, page_size, timerange)
    }

    /// Get a page of comments from a source.
    pub fn get_emails_iter_page(
        &self,
        bucket_name: &BucketFullName,
        continuation: Option<&EmailContinuation>,
        limit: usize,
    ) -> Result<EmailsIterPage> {
        let query_params = GetEmailsIterPageQuery {
            continuation,
            limit,
        };
        self.post(
            self.endpoints.get_emails(bucket_name)?,
            Some(&query_params),
            Retry::Yes,
        )
    }

    /// Iterate through all comments in a source.
    pub fn get_emails_iter<'a>(
        &'a self,
        bucket_name: &'a BucketFullName,
        page_size: Option<usize>,
    ) -> EmailsIter<'a> {
        EmailsIter::new(self, bucket_name, page_size)
    }

    /// Get a single comment by id.
    pub fn get_comment<'a>(
        &'a self,
        source_name: &'a SourceFullName,
        comment_id: &'a CommentId,
    ) -> Result<Comment> {
        let query_params = GetCommentQuery {
            include_markup: true,
        };
        Ok(self
            .get_query::<_, _, GetCommentResponse>(
                self.endpoints.comment_by_id(source_name, comment_id)?,
                Some(&query_params),
            )?
            .comment)
    }
    pub fn post_integration(
        &self,
        name: &IntegrationFullName,
        integration: &NewIntegration,
    ) -> Result<PostIntegrationResponse> {
        self.request(
            Method::POST,
            self.endpoints.integration(name)?,
            Some(PostIntegrationRequest {
                integration: integration.clone(),
            }),
            None::<()>,
            Retry::No,
        )
    }

    pub fn put_integration(
        &self,
        name: &IntegrationFullName,
        integration: &NewIntegration,
    ) -> Result<PutIntegrationResponse> {
        self.request(
            Method::PUT,
            self.endpoints.integration(name)?,
            Some(PutIntegrationRequest {
                integration: integration.clone(),
            }),
            None::<()>,
            Retry::No,
        )
    }

    pub fn put_comments(
        &self,
        source_name: &SourceFullName,
        comments: &[NewComment],
        no_charge: bool,
    ) -> Result<PutCommentsResponse> {
        self.request(
            Method::PUT,
            self.endpoints.put_comments(source_name)?,
            Some(PutCommentsRequest { comments }),
    