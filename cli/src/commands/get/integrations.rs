use std::{fs::File, io::BufWriter, path::PathBuf};

use anyhow::{Context, Result};
use reinfer_client::{resources::integration::Integration, Client, IntegrationFullName};
use structopt::StructOpt;

use crate::printer::{print_resources_as_json, Printer};

#[derive(Debug, StructOpt)]
pub struct GetIntegrationsArgs {
    #[structopt(name = "name")]
    /// The full name of the integration to get
    name: Option<IntegrationFullName>,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to 