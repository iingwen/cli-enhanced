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
}

pub fn get(client: &Client, args: &GetSourcesArgs, printer: &Printer) -> Result<()> {
    let GetSourcesArgs {
        source,
        include_stats,
    } = args;

    let sources = if let Some(source) = source {
        vec![client
            .get_source(source.clone())
            .context("Operation to list sources has failed.")?]
    } else {
        let mut sources = client
            .get_sources()
            .context("Operation to list sources has failed.")?;
        sources.sort_unstable_by(|lhs, rhs| {
            (&lhs.owner.0, &lhs