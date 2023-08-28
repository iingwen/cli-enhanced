use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{Client, ProjectName, UpdateProject};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct UpdateProjectArgs {
    #[structopt(name = "project-name")]
    /// Full name of the project
    name: ProjectName,

    #[structopt(long = "title")]
    /// Set the title of the project
    title: Option<String>,

    #[structopt(long = "description")]
    /// Set the description of the project
    description: Option<String>,
}

pub fn update(client: &Client, args: &UpdateProjectArgs, printer: &Printer) -> Result<()> {
    let UpdateProjectArgs {
        name,
        title,
        description,
    } = args;

    let project = client
        .update_project(
            name,
            UpdateProject {
