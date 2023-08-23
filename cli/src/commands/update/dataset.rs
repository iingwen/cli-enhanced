use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{Client, DatasetIdentifier, SourceId, SourceIdentifier, UpdateDataset};
use structopt::StructOpt;

/// Update a dataset.
#[derive(Debug, StructOpt)]
pub struct UpdateDatasetArgs {
    #[structopt(name = "dataset")]
    /// Name or id of the dataset to delete
    dataset: DatasetIdentifier,

    #[structopt(long = "title")]
    /// Set the title of the dataset
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the dataset
    description: Option<String>,

    #[structopt(short = "s", long = "source")]
    /// Names or ids of the sources in the dataset
    sources: Option<Vec<SourceIdentifier>>,
}

pub fn update(client: &Client, args: &UpdateD