use anyhow::{Context, Result};
use log::info;
use reinfer_client::{resources::source::StatisticsRequestParams, Client, SourceIdentifier};
use std::collections::HashMap;
use structopt::StructOpt;

use crate::printer::{PrintableSource, Printer};

#[derive(Debug, StructOpt)]
pub struct GetSourcesArgs {
    #[structopt(name = "source")]
    /// If specified, only list this source (name or id)
    source: Option<SourceIdentifier>,

    #[structopt(long = "stats")]
    /// Whether to include source statistics in response
    include_stats: bool,