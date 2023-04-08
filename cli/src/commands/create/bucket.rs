use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketFullName, BucketType, Client, NewBucket};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateBucketArgs {
    #[structopt(name = "bucket-name")]
    /// Full name of the new bucket <owner>/<name>
    name: BucketFullName,

    #[structopt(long = "title")]
    /// Set the title of the new bucket
    title: Option<String>,

    #[structopt(default_value, long = "type")]
    /// Set the type of the new bucket. Currently,