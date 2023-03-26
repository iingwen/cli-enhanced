
use crate::progress::{Options as ProgressOptions, Progress};
use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use reinfer_client::{
    resources::comment::{should_skip_serializing_optional_vec, EitherLabelling, HasAnnotations},
    Client, CommentId, CommentUid, DatasetFullName, DatasetIdentifier, NewEntities, NewLabelling,
    NewMoonForm, Source, SourceIdentifier,
};
use scoped_threadpool::Pool;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::channel;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateAnnotationsArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with annotations. If not specified, stdin will be used.
    annotations_path: Option<PathBuf>,

    #[structopt(short = "s", long = "source")]
    /// Name or id of the source containing the annotated comments
    source: SourceIdentifier,

    #[structopt(short = "d", long = "dataset")]
    /// Dataset (name or id) where to push the annotations. The dataset must contain the source.
    dataset: DatasetIdentifier,

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,

    #[structopt(long)]
    /// Whether to use the moon_forms field when creating annotations
    /// for a comment.
    use_moon_forms: bool,

    #[structopt(long = "batch-size", default_value = "128")]
    /// Number of comments to batch in a single request.
    batch_size: usize,
}

pub fn create(client: &Client, args: &CreateAnnotationsArgs, pool: &mut Pool) -> Result<()> {
    let source = client
        .get_source(args.source.clone())
        .with_context(|| format!("Unable to get source {}", args.source))?;
    let source_name = source.full_name();