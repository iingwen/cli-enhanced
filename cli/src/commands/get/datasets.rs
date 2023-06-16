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
    dataset: Opti