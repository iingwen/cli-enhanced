use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketIdentifier, Client, SourceIdentifier, TransformTag, UpdateSource};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UpdateSourceArgs {
    #[structopt(