use anyhow::{Context, Result};
use reinfer_client::{Client, ProjectName};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetProjectsArgs {
    #[structopt(name = "project")]