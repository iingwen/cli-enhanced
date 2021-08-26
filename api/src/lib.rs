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
            Some(NoChargeQuery { no_charge }),
            Retry::No,
        )
    }

    pub fn put_stream(
        &self,
        dataset_name: &DatasetFullName,
        stream: &NewStream,
    ) -> Result<PutStreamResponse> {
        self.put(
            self.endpoints.streams(dataset_name)?,
            Some(PutStreamRequest { stream }),
        )
    }

    pub fn get_audit_events(
        &self,
        minimum_timestamp: Option<DateTime<Utc>>,
        maximum_timestamp: Option<DateTime<Utc>>,
        continuation: Option<Continuation>,
    ) -> Result<AuditQueryResponse> {
        self.post::<_, _, AuditQueryResponse>(
            self.endpoints.audit_events_query()?,
            AuditQueryRequest {
                continuation,
                filter: AuditQueryFilter {
                    timestamp: CommentTimestampFilter {
                        minimum: minimum_timestamp,
                        maximum: maximum_timestamp,
                    },
                },
            },
            Retry::Yes,
        )
    }

    pub fn get_validation(
        &self,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
    ) -> Result<ValidationResponse> {
        self.get::<_, ValidationResponse>(self.endpoints.validation(dataset_name, model_version)?)
    }

    pub fn get_label_validation(
        &self,
        label: &LabelName,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
    ) -> Result<LabelValidation> {
        Ok(self
            .post::<_, _, LabelValidationResponse>(
                self.endpoints
                    .label_validation(dataset_name, model_version)?,
                LabelValidationRequest {
                    label: label.clone(),
                },
                Retry::Yes,
            )?
            .label_validation)
    }

    pub fn sync_comments(
        &self,
        source_name: &SourceFullName,
        comments: &[NewComment],
        no_charge: bool,
    ) -> Result<SyncCommentsResponse> {
        self.request(
            Method::POST,
            self.endpoints.sync_comments(source_name)?,
            Some(SyncCommentsRequest { comments }),
            Some(NoChargeQuery { no_charge }),
            Retry::Yes,
        )
    }

    pub fn sync_raw_emails(
        &self,
        source_name: &SourceFullName,
        documents: &[Document],
        transform_tag: &TransformTag,
        include_comments: bool,
        no_charge: bool,
    ) -> Result<SyncRawEmailsResponse> {
        self.request(
            Method::POST,
            self.endpoints.sync_comments_raw_emails(source_name)?,
            Some(SyncRawEmailsRequest {
                documents,
                transform_tag,
                include_comments,
            }),
            Some(NoChargeQuery { no_charge }),
            Retry::Yes,
        )
    }

    pub fn put_emails(
        &self,
        bucket_name: &BucketFullName,
        emails: &[NewEmail],
        no_charge: bool,
    ) -> Result<PutEmailsResponse> {
        self.request(
            Method::PUT,
            self.endpoints.put_emails(bucket_name)?,
            Some(PutEmailsRequest { emails }),
            Some(NoChargeQuery { no_charge }),
            Retry::Yes,
        )
    }

    pub fn post_user(&self, user_id: &UserId, user: UpdateUser) -> Result<PostUserResponse> {
        self.post(
            self.endpoints.post_user(user_id)?,
            PostUserRequest { user: &user },
            Retry::Yes,
        )
    }

    pub fn put_comment_audio(
        &self,
        source_id: &SourceId,
        comment_id: &CommentId,
        audio_path: impl AsRef<Path>,
    ) -> Result<()> {
        let form = Form::new()
            .file("file", audio_path)
            .map_err(|source| Error::Unknown {
                message: "PUT comment audio operation failed".to_owned(),
                source: source.into(),
            })?;
        let http_response = self
            .http_client
            .put(self.endpoints.comment_audio(source_id, comment_id)?)
            .headers(self.headers.clone())
            .multipart(form)
            .send()
            .map_err(|source| Error::ReqwestError {
                message: "PUT comment audio operation failed".to_owned(),
                source,
            })?;
        let status = http_response.status();
        http_response
            .json::<Response<EmptySuccess>>()
            .map_err(Error::BadJsonResponse)?
            .into_result(status)?;
        Ok(())
    }

    pub fn get_integrations(&self) -> Result<Vec<Integration>> {
        Ok(self
            .get::<_, GetIntegrationsResponse>(self.endpoints.integrations()?)?
            .integrations)
    }

    pub fn get_integration(&self, name: &IntegrationFullName) -> Result<Integration> {
        Ok(self
            .get::<_, GetIntegrationResponse>(self.endpoints.integration(name)?)?
            .integration)
    }

    pub fn get_datasets(&self) -> Result<Vec<Dataset>> {
        Ok(self
            .get::<_, GetAvailableDatasetsResponse>(self.endpoints.datasets.clone())?
            .datasets)
    }

    pub fn get_dataset<IdentifierT>(&self, dataset: IdentifierT) -> Result<Dataset>
    where
        IdentifierT: Into<DatasetIdentifier>,
    {
        Ok(match dataset.into() {
            DatasetIdentifier::Id(dataset_id) => {
                self.get::<_, GetDatasetResponse>(self.endpoints.dataset_by_id(&dataset_id)?)?
                    .dataset
            }
            DatasetIdentifier::FullName(dataset_name) => {
                self.get::<_, GetDatasetResponse>(self.endpoints.dataset_by_name(&dataset_name)?)?
                    .dataset
            }
        })
    }

    /// Create a dataset.
    pub fn create_dataset(
        &self,
        dataset_name: &DatasetFullName,
        options: NewDataset<'_>,
    ) -> Result<Dataset> {
        Ok(self
            .put::<_, _, CreateDatasetResponse>(
                self.endpoints.dataset_by_name(dataset_name)?,
                CreateDatasetRequest { dataset: options },
            )?
            .dataset)
    }

    /// Update a dataset.
    pub fn update_dataset(
        &self,
        dataset_name: &DatasetFullName,
        options: UpdateDataset<'_>,
    ) -> Result<Dataset> {
        Ok(self
            .post::<_, _, UpdateDatasetResponse>(
                self.endpoints.dataset_by_name(dataset_name)?,
                UpdateDatasetRequest { dataset: options },
                Retry::Yes,
            )?
            .dataset)
    }

    pub fn delete_dataset<IdentifierT>(&self, dataset: IdentifierT) -> Result<()>
    where
        IdentifierT: Into<DatasetIdentifier>,
    {
        let dataset_id = match dataset.into() {
            DatasetIdentifier::Id(dataset_id) => dataset_id,
            dataset @ DatasetIdentifier::FullName(_) => self.get_dataset(dataset)?.id,
        };
        self.delete(self.endpoints.dataset_by_id(&dataset_id)?)
    }

    /// Get labellings for a given a dataset and a list of comment UIDs.
    pub fn get_labellings<'a>(
        &self,
        dataset_name: &DatasetFullName,
        comment_uids: impl Iterator<Item = &'a CommentUid>,
    ) -> Result<Vec<AnnotatedComment>> {
        Ok(self
            .get_query::<_, _, GetAnnotationsResponse>(
                self.endpoints.get_labellings(dataset_name)?,
                Some(&id_list_query(comment_uids.into_iter().map(|id| &id.0))),
            )?
            .results)
    }

    /// Iterate through all reviewed comments in a source.
    pub fn get_labellings_iter<'a>(
        &'a self,
        dataset_name: &'a DatasetFullName,
        source_id: &'a SourceId,
        return_predictions: bool,
        limit: Option<usize>,
    ) -> LabellingsIter<'a> {
        LabellingsIter::new(self, dataset_name, source_id, return_predictions, limit)
    }

    /// Get reviewed comments in bulk
    pub fn get_labellings_in_bulk(
        &self,
        dataset_name: &DatasetFullName,
        query_parameters: GetLabellingsInBulk<'_>,
    ) -> Result<GetAnnotationsResponse> {
        self.get_query::<_, _, GetAnnotationsResponse>(
            self.endpoints.get_labellings(dataset_name)?,
            Some(&query_parameters),
        )
    }

    /// Update labellings for a given a dataset and comment UID.
    pub fn update_labelling(
        &self,
        dataset_name: &DatasetFullName,
        comment_uid: &CommentUid,
        labelling: Option<&[NewLabelling]>,
        entities: Option<&NewEntities>,
        moon_forms: Option<&[NewMoonForm]>,
    ) -> Result<AnnotatedComment> {
        self.post::<_, _, AnnotatedComment>(
            self.endpoints.post_labelling(dataset_name, comment_uid)?,
            UpdateAnnotationsRequest {
                labelling,
                entities,
                moon_forms,
            },
            Retry::Yes,
        )
    }

    /// Get predictions for a given a dataset, a model version, and a list of comment UIDs.
    pub fn get_comment_predictions<'a>(
        &self,
        dataset_name: &DatasetFullName,
        model_version: &ModelVersion,
        comment_uids: impl Iterator<Item = &'a CommentUid>,
    ) -> Result<Vec<Prediction>> {
        Ok(self
            .post::<_, _, GetPredictionsResponse>(
                self.endpoints
                    .get_comment_predictions(dataset_name, model_version)?,
                json!({
                    "threshold": "auto",
                    "uids": comment_uids.into_iter().map(|id| id.0.as_str()).collect::<Vec<_>>(),
                }),
                Retry::Yes,
            )?
            .predictions)
    }

    pub fn get_streams(&self, dataset_name: &DatasetFullName) -> Result<Vec<Stream>> {
        Ok(self
            .get::<_, GetStreamsResponse>(self.endpoints.streams(dataset_name)?)?
            .streams)
    }

    pub fn get_recent_comments(
        &self,
        dataset_name: &DatasetFullName,
        filter: &CommentFilter,
        limit: usize,
        continuation: Option<&Continuation>,
    ) -> Result<RecentCommentsPage> {
        self.post::<_, _, RecentCommentsPage>(
            self.endpoints.recent_comments(dataset_name)?,
            GetRecentRequest {
                limit,
                filter,
                continuation,
            },
            Retry::No,
        )
    }

    pub fn get_current_user(&self) -> Result<User> {
        Ok(self
            .get::<_, GetCurrentUserResponse>(self.endpoints.current_user.clone())?
            .user)
    }

    pub fn get_users(&self) -> Result<Vec<User>> {
        Ok(self
            .get::<_, GetAvailableUsersResponse>(self.endpoints.users.clone())?
            .users)
    }

    pub fn create_user(&self, user: NewUser<'_>) -> Result<User> {
        Ok(self
            .put::<_, _, CreateUserResponse>(
                self.endpoints.users.clone(),
                CreateUserRequest { user },
            )?
            .user)
    }

    pub fn dataset_summary(
        &self,
        dataset_name: &DatasetFullName,
        params: &SummaryRequestParams,
    ) -> Result<SummaryResponse> {
        self.post::<_, _, SummaryResponse>(
            self.endpoints.dataset_summary(dataset_name)?,
            serde_json::to_value(params).expect("summary params serialization error"),
            Retry::Yes,
        )
    }

    pub fn query_dataset(
        &self,
        dataset_name: &DatasetFullName,
        params: &QueryRequestParams,
    ) -> Result<QueryResponse> {
        self.post::<_, _, QueryResponse>(
            self.endpoints.query_dataset(dataset_name)?,
            serde_json::to_value(params).expect("query params serialization error"),
            Retry::Yes,
        )
    }

    pub fn send_welcome_email(&self, user_id: UserId) -> Result<()> {
        self.post::<_, _, WelcomeEmailResponse>(
            self.endpoints.welcome_email(&user_id)?,
            json!({}),
            Retry::No,
        )?;
        Ok(())
    }

    pub fn get_bucket_statistics(&self, bucket_name: &BucketFullName) -> Result<BucketStatistics> {
        Ok(self
            .get::<_, GetBucketStatisticsResponse>(self.endpoints.bucket_statistics(bucket_name)?)?
            .statistics)
    }

    pub fn get_dataset_statistics(
        &self,
        dataset_name: &DatasetFullName,
        params: &DatasetStatisticsRequestParams,
    ) -> Result<CommentStatistics> {
        Ok(self
            .post::<_, _, GetStatisticsResponse>(
                self.endpoints.dataset_statistics(dataset_name)?,
                serde_json::to_value(params)
                    .expect("dataset statistics params serialization error"),
                Retry::No,
            )?
            .statistics)
    }

    pub fn get_source_statistics(
        &self,
        source_name: &SourceFullName,
        params: &SourceStatisticsRequestParams,
    ) -> Result<CommentStatistics> {
        Ok(self
            .post::<_, _, GetStatisticsResponse>(
                self.endpoints.source_statistics(source_name)?,
                serde_json::to_value(params).expect("source statistics params serialization error"),
                Retry::No,
            )?
            .statistics)
    }

    /// Create a new bucket.
    pub fn create_bucket(
        &self,
        bucket_name: &BucketFullName,
        options: NewBucket<'_>,
    ) -> Result<Bucket> {
        Ok(self
            .put::<_, _, CreateBucketResponse>(
                self.endpoints.bucket_by_name(bucket_name)?,
                CreateBucketRequest { bucket: options },
            )?
            .bucket)
    }

    pub fn get_buckets(&self) -> Result<Vec<Bucket>> {
        Ok(self
            .get::<_, GetAvailableBucketsResponse>(self.endpoints.buckets.clone())?
            .buckets)
    }

    pub fn get_bucket<IdentifierT>(&self, bucket: IdentifierT) -> Result<Bucket>
    where
        IdentifierT: Into<BucketIdentifier>,
    {
        Ok(match bucket.into() {
            BucketIdentifier::Id(bucket_id) => {
                self.get::<_, GetBucketResponse>(self.endpoints.bucket_by_id(&bucket_id)?)?
                    .bucket
            }
            BucketIdentifier::FullName(bucket_name) => {
                self.get::<_, GetBucketResponse>(self.endpoints.bucket_by_name(&bucket_name)?)?
                    .bucket
            }
        })
    }

    pub fn delete_bucket<IdentifierT>(&self, bucket: IdentifierT) -> Result<()>
    where
        IdentifierT: Into<BucketIdentifier>,
    {
        let bucket_id = match bucket.into() {
            BucketIdentifier::Id(bucket_id) => bucket_id,
            bucket @ BucketIdentifier::FullName(_) => self.get_bucket(bucket)?.id,
        };
        self.delete(self.endpoints.bucket_by_id(&bucket_id)?)
    }

    pub fn fetch_stream_comments(
        &self,
        stream_name: &StreamFullName,
        size: u32,
    ) -> Result<StreamBatch> {
        self.post(
            self.endpoints.stream_fetch(stream_name)?,
            StreamFetchRequest { size },
            Retry::No,
        )
    }

    pub fn get_stream(&self, stream_name: &StreamFullName) -> Result<Stream> {
        Ok(self
            .get::<_, GetStreamResponse>(self.endpoints.stream(stream_name)?)?
            .stream)
    }

    pub fn advance_stream(
        &self,
        stream_name: &StreamFullName,
        sequence_id: StreamSequenceId,
    ) -> Result<()> {
        self.post::<_, _, serde::de::IgnoredAny>(
            self.endpoints.stream_advanc