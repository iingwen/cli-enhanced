use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketFullName, BucketType, Client, NewBucket};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateBucketArgs {
    #[s