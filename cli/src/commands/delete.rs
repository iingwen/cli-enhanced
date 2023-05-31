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
 