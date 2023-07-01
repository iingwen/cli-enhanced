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
    d