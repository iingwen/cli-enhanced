
mod audit_events;
mod buckets;
mod comments;
mod datasets;
mod emails;
mod integrations;
mod projects;
mod quota;
mod sources;
mod streams;
mod users;

use anyhow::Result;
use reinfer_client::Client;
use scoped_threadpool::Pool;
use structopt::StructOpt;

use self::{
    audit_events::GetAuditEventsArgs,
    buckets::GetBucketsArgs,
    comments::{GetManyCommentsArgs, GetSingleCommentArgs},
    datasets::GetDatasetsArgs,
    emails::GetManyEmailsArgs,
    integrations::GetIntegrationsArgs,
    projects::GetProjectsArgs,
    sources::GetSourcesArgs,
    streams::{GetStreamCommentsArgs, GetStreamStatsArgs, GetStreamsArgs},
    users::GetUsersArgs,
};
use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub enum GetArgs {
    #[structopt(name = "buckets")]
    /// List the available buckets
    Buckets(GetBucketsArgs),

    #[structopt(name = "emails")]
    /// Download all emails from a source
    Emails(GetManyEmailsArgs),

    #[structopt(name = "comment")]
    /// Get a single comment from a source
    Comment(GetSingleCommentArgs),

    #[structopt(name = "comments")]
    /// Download all comments from a source
    Comments(GetManyCommentsArgs),

    #[structopt(name = "datasets")]
    /// List the available datasets
    Datasets(GetDatasetsArgs),

    #[structopt(name = "projects")]
    /// List the available projects
    Projects(GetProjectsArgs),

    #[structopt(name = "sources")]
    /// List the available sources
    Sources(GetSourcesArgs),

    #[structopt(name = "streams")]
    /// List the available streams for a dataset
    Streams(GetStreamsArgs),

    #[structopt(name = "stream-comments")]
    /// Fetch comments from a stream
    StreamComments(GetStreamCommentsArgs),
