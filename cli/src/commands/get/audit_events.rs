use anyhow::Result;
use chrono::{DateTime, Utc};
use log::info;
use reinfer_client::{resources::audit::PrintableAuditEvent, Client};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetAuditEventsArgs {
    #[structopt(short = "m", long = "minimum")]
    /// Minimum Timestamp for audit events
    minimum_timestamp: Option<DateTime<Utc>>,

    #[structopt(short = "M", long = "maximum")]
    /