
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

    #[structopt(name = "stream-stats")]
    /// Get the validation stats for a given stream
    StreamStats(GetStreamStatsArgs),

    #[structopt(name = "users")]
    /// List the available users
    Users(GetUsersArgs),

    #[structopt(name = "current-user")]
    /// Get the user associated with the API token in use
    CurrentUser,

    #[structopt(name = "quotas")]
    /// List all quotas for current tenant
    Quotas,

    #[structopt(name = "audit-events")]
    /// Get audit events for current tenant
    AuditEvents(GetAuditEventsArgs),

    #[structopt(name = "integrations")]
    /// Get integrations
    Integrations(GetIntegrationsArgs),
}

pub fn run(args: &GetArgs, client: Client, printer: &Printer, pool: &mut Pool) -> Result<()> {
    match args {
        GetArgs::Buckets(args) => buckets::get(&client, args, printer),
        GetArgs::Emails(args) => emails::get_many(&client, args),
        GetArgs::Comment(args) => comments::get_single(&client, args),
        GetArgs::Comments(args) => comments::get_many(&client, args),
        GetArgs::Datasets(args) => datasets::get(&client, args, printer),
        GetArgs::Projects(args) => projects::get(&client, args, printer),
        GetArgs::Sources(args) => sources::get(&client, args, printer),
        GetArgs::Streams(args) => streams::get(&client, args, printer),
        GetArgs::StreamComments(args) => streams::get_stream_comments(&client, args),
        GetArgs::StreamStats(args) => streams::get_stream_stats(&client, args, printer, pool),
        GetArgs::Users(args) => users::get(&client, args, printer),
        GetArgs::CurrentUser => users::get_current_user(&client, printer),
        GetArgs::Quotas => quota::get(&client, printer),
        GetArgs::AuditEvents(args) => audit_events::get(&client, args, printer),
        GetArgs::Integrations(args) => integrations::get(&client, args, printer),
    }
}