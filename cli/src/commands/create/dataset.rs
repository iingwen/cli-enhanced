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

    #[structopt(long = "description")