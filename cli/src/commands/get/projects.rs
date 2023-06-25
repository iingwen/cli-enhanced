use anyhow::{Context, Result};
use reinfer_client::{Client, ProjectName};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetProjectsArgs {
    #[structopt(name = "project")]
    /// If specified, only list this project (name or id)
    project: Option<ProjectName>,
}

pub fn get(client: &Client, args: &GetProjectsArgs, printer: &Printer) -> Result<()> {
    let GetProjectsArgs { project } = args;
    let projects = if let Some(project) = project {
       