use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{Client, ProjectName, UpdateProject};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UpdateProjectArgs {
    #[structo