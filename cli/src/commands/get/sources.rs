use anyhow::{Context, Result};
use log::info;
use reinfer_client::{resources::source::StatisticsRequestParams, Client, SourceIdentifier};
use std::collections::HashMap;
use structopt::StructOpt;

use crate::printer::{PrintableSource, Printer};

#[derive(Debug, StructOpt)]
