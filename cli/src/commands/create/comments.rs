
use crate::{
    commands::{
        create::annotations::{
            upload_batch_of_annotations, AnnotationStatistic, CommentIdComment, NewAnnotation,
        },
        ensure_uip_user_consents_to_ai_unit_charge,
    },
    progress::{Options as ProgressOptions, Progress},
};
use anyhow::{anyhow, ensure, Context, Result};
use colored::Colorize;
use log::{debug, info};
use reinfer_client::{
    Client, CommentId, DatasetFullName, DatasetIdentifier, NewAnnotatedComment, NewComment, Source,
    SourceIdentifier,
};
use scoped_threadpool::Pool;
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufRead, BufReader, Seek},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateCommentsArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with comments. If not specified, stdin will be used.
    comments_path: Option<PathBuf>,

    #[structopt(short = "s", long = "source")]
    /// Name or id of the source where the comments will be uploaded.
    source: SourceIdentifier,

    #[structopt(short = "d", long = "dataset")]
    /// Optionally, a dataset (name or id) where to push the annotations. The
    /// dataset must contain the source.
    dataset: Option<DatasetIdentifier>,

    #[structopt(long = "batch-size", default_value = "128")]
    /// Number of comments to batch in a single request.
    batch_size: usize,

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,

    #[structopt(long)]
    /// Whether to allow duplicate comment IDs in the input.
    allow_duplicates: bool,

    #[structopt(long)]
    /// Whether to allow overwriting existing comments in the source.
    ///
    /// If not set, the upload will halt when encountering a comment with an ID
    /// which is already associated to different data on the platform.
    overwrite: bool,

    #[structopt(long)]
    /// Whether to use the moon_forms field when creating annotations
    /// for a comment.
    use_moon_forms: bool,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to ai unit charge. Suppresses confirmation prompt.
    yes: bool,
}

pub fn create(client: &Client, args: &CreateCommentsArgs, pool: &mut Pool) -> Result<()> {
    if !args.no_charge && !args.yes {
        ensure_uip_user_consents_to_ai_unit_charge(client.base_url())?;
    }

    let source = client
        .get_source(args.source.clone())
        .with_context(|| format!("Unable to get source {}", args.source))?;

    let source_name = source.full_name();

    let dataset_name = match args.dataset.as_ref() {
        Some(dataset_ident) => Some(
            client
                .get_dataset(dataset_ident.clone())
                .with_context(|| format!("Unable to get dataset {}", args.source))?
                .full_name(),
        ),
        None => None,
    };

    ensure!(args.batch_size > 0, "--batch-size must be greater than 0");

    let statistics = match &args.comments_path {
        Some(comments_path) => {
            info!(
                "Uploading comments from file `{}` to source `{}` [id: {}]",
                comments_path.display(),
                source_name.0,
                source.id.0,
            );
            let mut file =
                BufReader::new(File::open(comments_path).with_context(|| {
                    format!("Could not open file `{}`", comments_path.display())
                })?);
            let file_metadata = file.get_ref().metadata().with_context(|| {
                format!(
                    "Could not get file metadata for `{}`",
                    comments_path.display()
                )
            })?;

            if !args.allow_duplicates {
                debug!(
                    "Checking `{}` for duplicate comment ids",
                    comments_path.display(),
                );
                check_no_duplicate_ids(&mut file)?;

                file.rewind().with_context(|| {
                    "Unable to seek to file start after checking for duplicate ids"
                })?;
            }

            let statistics = Arc::new(Statistics::new());
            let progress = if args.no_progress {
                None
            } else {
                Some(progress_bar(
                    file_metadata.len(),
                    &statistics,
                    args.overwrite,
                ))
            };
            upload_comments_from_reader(
                client,
                &source,
                file,
                args.batch_size,
                &statistics,
                dataset_name.as_ref(),
                args.overwrite,
                args.allow_duplicates,
                args.use_moon_forms,
                args.no_charge,
                pool,
            )?;
            if let Some(mut progress) = progress {
                progress.done();
            }
            Arc::try_unwrap(statistics).unwrap()
        }
        None => {
            info!(
                "Uploading comments from stdin to source `{}` [id: {}]",
                source_name.0, source.id.0,
            );
            ensure!(
                args.allow_duplicates,
                "--allow-duplicates is required when uploading from stdin"
            );
            let statistics = Statistics::new();
            upload_comments_from_reader(
                client,