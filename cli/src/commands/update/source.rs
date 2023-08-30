use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{BucketIdentifier, Client, SourceIdentifier, TransformTag, UpdateSource};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UpdateSourceArgs {
    #[structopt(name = "source")]
    /// Id or full name of the source to update
    source: SourceIdentifier,

    #[structopt(long = "title")]
    /// Set the title of the source
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the source
    description: Option<String>,

    #[structopt(long = "should-translate")]
    /// Enable translation for the source
    should_translate: Option<bool>,

    #[structopt(long = "bucket")]
    /// Bucket to pull emails from.
    bucket: Option<BucketIdentifier>,

    #[structopt(long = "transform-tag")]
    /// Set the transform tag of the source
    transform_tag: Option<TransformTag>,
}

pub fn update(client: &Client, args: &UpdateSourceArgs, printer: &Printer) -> Result<()> {
    let UpdateSourceArgs {
        source,
        title,
        description,
        should_translate,
        bucket,
        transform_tag,
    } = args;

    let bucket_id = match bucket.to_owned() {
        Some(BucketIdentifier::Id(bucket_id)) => Some(bucket_id),
        Some(full_name @ BucketIdentifier::FullName(_)) => Some(
            client
            