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
        CreateRequest as CreateBucketRequest, CreateResp