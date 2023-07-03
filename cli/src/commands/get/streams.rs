use anyhow::{anyhow, Context, Result};
use colored::{ColoredString, Colorize};
use log::info;
use ordered_float::NotNan;
use prettytable::row;
use reinfer_client::resources::stream::{StreamLabelThreshold, StreamModel};
use reinfer_client::resources::validation::ValidationResponse;
use reinfer_client::{
    resources::validation::LabelValidation, Client, DatasetIdentifier, ModelVersion, StreamFullName,
};
use reinfer_client::{DatasetFullName, LabelDef, LabelName};
use scoped_threadpool::Pool;
use serde::Serialize;
use std::sync::mpsc::channel;
use std::{
    fs::File,
    io,
    io::{BufWriter, Write},
    path::PathBuf,
};
use structopt::StructOpt;

use crate::printer::{print_resources_as_json, DisplayTable, Printer};

#[derive(Debug, StructOpt)]
pub struct GetStreamsArgs {
    #[structopt(short = "d", long = "dataset")]
    /// The dataset name or id
    dataset: DatasetIdentifier,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write streams as JSON.
    path: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct GetStreamCommentsArgs {
    #[structopt(long = "stream")]
    /// The full stream name `<owner>/<dataset>/<stream>`.
    stream: StreamFullName,

    #[structopt(long = "size", default_value = "16")]
    /// The max number of comments to return per batch.
    size: u32,

    #[structopt(long = "listen")]
    /// If set, the command will run forever polling every N seconds and advancing the stream.
    listen: Option<f64>,

    #[structopt(long = "individual-advance")]
    /// If set, the command will acknowledge each comment in turn, rather than full batches.
    individual_advance: bool,
}

#[derive(Debug, StructOpt)]
pub struct GetStreamStatsArgs {
    #[structopt(name = "stream")]
    /// The full stream name `<owner>/<dataset>/<stream>`.
    stream_full_name: StreamFullName,

    #[structopt(long = "compare-version", short = "v")]
    /// The model version to compare stats with
    compare_to_model_version: Option<ModelVersion>,

    #[structopt(long = "compare-dataset", short = "d")]
    /// The dataset to compare stats with
    compare_to_dataset: Option<DatasetFullName>,
}

pub fn get(client: &Client, args: &GetStreamsArgs, printer: &Printer) -> 