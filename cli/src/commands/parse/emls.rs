
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use log::error;
use mailparse::{DispositionType, MailHeader, MailHeaderMap};
use scoped_threadpool::Pool;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{mpsc::channel, Arc},
};

use crate::commands::{
    ensure_uip_user_consents_to_ai_unit_charge,
    parse::{get_files_in_directory, get_progress_bar, Statistics},
};
use reinfer_client::{resources::email::AttachmentMetadata, BucketIdentifier, Client, NewEmail};
use structopt::StructOpt;

use super::upload_batch_of_new_emails;
const UPLOAD_BATCH_SIZE: usize = 4;

#[derive(Debug, StructOpt)]
pub struct ParseEmlArgs {
    #[structopt(short = "d", long = "dir", parse(from_os_str))]
    /// Directory containing the emls
    directory: PathBuf,

    #[structopt(short = "b", long = "bucket")]
    /// Name of the bucket where the emails will be uploaded.
    bucket: BucketIdentifier,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to ai unit charge. Suppresses confirmation prompt.
    yes: bool,
}

pub fn parse(client: &Client, args: &ParseEmlArgs, pool: &mut Pool) -> Result<()> {
    let ParseEmlArgs {
        directory,
        bucket,
        no_charge,
        yes,
    } = args;

    if !no_charge && !yes {
        ensure_uip_user_consents_to_ai_unit_charge(client.base_url())?;
    }
