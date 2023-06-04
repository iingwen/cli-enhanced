use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use structopt::StructOpt;

use reinfer_client::{
    resources::project::ForceDeleteProject, BucketIdentifier, Client, CommentId, CommentsIter,
    CommentsIterTimerange, DatasetIdentifier, ProjectName, Source, SourceIdentifier,
    UserIdentifier,
};

use crate::progress::{Options as ProgressOptions, Progress};

#[derive(Debug, StructOpt)]
pub enum DeleteArgs {
    #[structopt(name = "source")]
    /// Delete a source
    Source {
        #[structopt(name = "source")]
        /// Name or id of the source to delete
        source: SourceIdentifier,
    },

    #[structopt(name = "comments")]
    /// Delete comments by id in a source.
    Comments {
        #[structopt(short = "s", long = "source")]
        /// Name or id of the source to delete comments from
        source: SourceIdentifier,

        #[structopt(name = "comment id")]
        /// Ids of the comments to delete
        comments: Vec<CommentId>,
    },

    #[structopt(name = "bulk")]
    /// Delete all comments in a given time range.
    BulkComments {
        #[structopt(short = "s", long = "source")]
        /// Name or id of the source to delete comments from
        source: SourceIdentifier,

        #[structopt(long, parse(try_from_str))]
        /// Whether to delete comments that are annotated in any of the datasets
        /// containing this source.
        /// Use --include-annotated=false to keep any annotated comments in the given range.
        /// Use --include-annotated=true to delete all comments.
        include_annotated: bool,

        #[structopt(long)]
        /// Starting timestamp for comments to delete (inclusive). Should be in
        /// RFC 3339 format, e.g. 1970-01-02T03:04:05Z
        from_timestamp: Option<DateTime<Utc>>,

        #[structopt(long)]
        /// Ending timestamp for comments to delete (inclusive). Should be in
        /// RFC 3339 format, e.g. 1970-01-02T03:04:05Z
        to_timestamp: Option<DateTime<Utc>>,

        #[structopt(long)]
        /// Don't display a progress bar
        no_progress: bool,
    },

    #[structopt(name = "bucket")]
    /// Delete a bucket
    Bucket {
        #[structopt(name = "bucket")]
        /// Name or id of the bucket to delete
        bucket: BucketIdentifier,
    },

    #[structopt(name = "dataset")]
    /// Delete a dataset
    Dataset {
        #[structopt(name = "dataset")]
        /// Name or id of the dataset to delete
        dataset: DatasetIdentifier,
    },

    #[structopt(name = "user")]
    /// Delete a user
    User {
        #[structopt(name = "user")]
        /// Username or id of the user to delete
        user: UserIdentifier,
    },

    #[structopt(name = "project")]
    /// Delete a project
    Project {
        #[structopt(name = "project")]
        /// Name or id of the project to delete
        project: ProjectName,

        #[structopt(long)]
        /// Force deletion of the project, even if it's not empty.
        force: bool,
    },
}

pub fn run(delete_args: &DeleteArgs, client: Client) -> Result<()> {
    match delete_args {
        DeleteArgs::Source { source } => {
            client
                .delete_source(source.clone())
                .context("Operation to delete source has failed.")?;
            log::info!("Deleted source.");
        }
        DeleteArgs::User { user } => {
            client
                .delete_user(user.clone())
                .context("Operation to delete user has failed.")?;
            log::info!("Deleted user.");
        }
        DeleteArgs::Comments { source, comm