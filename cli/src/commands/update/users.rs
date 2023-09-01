
use anyhow::{Context, Result};
use colored::Colorize;
use log::info;
use reinfer_client::{Client, UpdateUser, UserId};
use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::progress::{Options as ProgressOptions, Progress};

#[derive(Debug, StructOpt)]
pub struct UpdateUsersArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with users. If not specified, stdin will be used.
    input_file: Option<PathBuf>,

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,
}

pub fn update(client: &Client, args: &UpdateUsersArgs) -> Result<()> {
    let statistics = match &args.input_file {
        Some(input_file) => {
            info!("Processing users from file `{}`", input_file.display(),);
            let file_metadata = fs::metadata(input_file).with_context(|| {
                format!("Could not get file metadata for `{}`", input_file.display())
            })?;
            let file = BufReader::new(
                File::open(input_file)
                    .with_context(|| format!("Could not open file `{}`", input_file.display()))?,
            );
            let statistics = Arc::new(Statistics::new());
            let progress = if args.no_progress {
                None
            } else {
                Some(progress_bar(file_metadata.len(), &statistics))
            };
            update_users_from_reader(client, file, &statistics)?;
            if let Some(mut progress) = progress {
                progress.done();
            }
            Arc::try_unwrap(statistics).unwrap()
        }
        None => {
            info!("Processing users from stdin",);
            let statistics = Statistics::new();
            update_users_from_reader(client, BufReader::new(io::stdin()), &statistics)?;
            statistics
        }
    };

    info!(
        concat!("Successfully updated {} users",),
        statistics.num_updated(),
    );

    Ok(())