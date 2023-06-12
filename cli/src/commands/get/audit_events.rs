use anyhow::Result;
use chrono::{DateTime, Utc};
use log::info;
use reinfer_client::{resources::audit::PrintableAuditEvent, Client};
use structopt::StructOpt;

use 