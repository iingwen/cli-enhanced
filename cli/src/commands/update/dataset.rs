use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{Client, DatasetIdentifier, SourceId, SourceIdentifier, UpdateDataset};
use structopt::StructOpt;

/// Update a dataset.
#[de