
mod emls;
mod msgs;

use anyhow::Result;
use colored::Colorize;
use reinfer_client::resources::bucket::FullName as BucketFullName;
use reinfer_client::resources::documents::Document;
use reinfer_client::{Client, NewEmail, Source, TransformTag};
use scoped_threadpool::Pool;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use structopt::StructOpt;

use crate::progress::{Options as ProgressOptions, Progress};

use self::emls::ParseEmlArgs;
use self::msgs::ParseMsgArgs;

#[derive(Debug, StructOpt)]
pub enum ParseArgs {
    #[structopt(name = "msgs")]
    /// Parse unicode msg files. Note: Currently the body is processed as plain text.
    /// Html bodies are not supported.
    Msgs(ParseMsgArgs),

    #[structopt(name = "emls")]
    /// Parse eml files.
    /// Html bodies are not supported.
    Emls(ParseEmlArgs),
}

pub fn run(args: &ParseArgs, client: Client, pool: &mut Pool) -> Result<()> {
    match args {
        ParseArgs::Msgs(parse_msg_args) => msgs::parse(&client, parse_msg_args),
        ParseArgs::Emls(parse_eml_args) => emls::parse(&client, parse_eml_args, pool),
    }
}

pub struct Statistics {
    processed: AtomicUsize,
    failed: AtomicUsize,
    uploaded: AtomicUsize,
}

impl Statistics {
    fn new() -> Self {
        Self {
            processed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            uploaded: AtomicUsize::new(0),
        }
    }

    #[inline]
    fn add_uploaded(&self, num_uploaded: usize) {
        self.uploaded.fetch_add(num_uploaded, Ordering::SeqCst);
    }

    #[inline]
    fn increment_failed(&self) {
        self.failed.fetch_add(1, Ordering::SeqCst);
    }

    #[inline]
    fn increment_processed(&self) {