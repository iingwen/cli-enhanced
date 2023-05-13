use crate::printer::Printer;
use anyhow::{anyhow, Context, Error, Result};
use log::info;
use reinfer_client::{
    Client, DatasetFullName, NewDataset, NewEntityDef, NewLabelDef, NewLabelGroup, SourceIdentifier,
};
use serde::Deserialize;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateDatasetArgs {
    #[structopt(name = "owner-name/dataset-name")]
    /// Full name of the new dataset <owner>/<name>
    name: DatasetFullName,

    #[structopt(long = "title")]
    /// Set the title of the new dataset
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the new dataset
    description: Option<String>,

    #[structopt(
        long = "has-sentiment",
        help = "Enable sentiment prediction for the dataset [default: false]"
    )]
    /// Enable sentiment prediction for the dataset
    has_sentiment: Option<bool>,

    #[structopt(short = "s", long = "source")]
    /// Names or ids of the sources in the dataset
    sources: Vec<SourceIdentifier>,

    #[structopt(short = "e", long = "entity-defs", default_value = "[]")]
    /// Entity defs to create at dataset creation, as json
    entity_defs: VecExt<NewEntityDef>,

    #[structopt(long = "label-defs", default_value = "[]")]
    /// Label defs to create at dataset creation, as json.
    /// Only used if label_groups is not provided.
    label_defs: VecExt<NewLabelDef>,

    #[structopt(long = "label-groups", default_value = "[]")]
    /// Label groups to create at dataset creation, as json
    label_groups: VecExt<NewLabelGroup>,

    #[structopt(long = "model-family")]
    /// Model family to use for the new dataset
    model_family: Option<String>,

    /// Dataset ID of the dataset to copy annotations from
    #[structopt(long = "copy-annotations-from")]
    copy_annotations_from: Option<String>,
}

pub fn create(client: &Client, args: &CreateDatasetArgs, printer: &Printer) -> Result<()> {
    let CreateDatasetArgs {
        name,
        title,
        description,
        has_sentiment,
        sources,
        entity_defs,
        label_defs,
        label_groups,
        model_family,
        copy_annotations_from,
    } = args;

    let source_ids = {
        let mut source_ids = Vec::with_capacity(sources.len());
        for source in sources.iter() {
            source_ids.push(
                client
                    .get_source(source.clone())
                    .context("Operation to get source has failed")?
                    .id,
            );
        