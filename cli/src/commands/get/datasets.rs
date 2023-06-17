use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::dataset::{DatasetAndStats, DatasetStats, StatisticsRequestParams},
    Client, CommentFilter, DatasetIdentifier,
};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetDatasetsArgs {
    #[structopt(name = "dataset")]
    /// If specified, only list this dataset (name or id)
    dataset: Option<DatasetIdentifier>,

    #[structopt(long = "stats")]
    /// Whether to include dataset statistics in response
    include_stats: bool,
}

pub fn get(client: &Client, args: &GetDatasetsArgs, printer: &Printer) -> Result<()> {
    let GetDatasetsArgs {
        dataset,
        include_stats,
    } = args;
    let datasets = if let Some(dataset) = dataset {
        vec![client
            .get_dataset(dataset.clone())
            .context("Operation to list datasets has failed.")?]
    } else {
        let mut datasets = client
            .get_datasets()
           