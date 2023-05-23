use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{Client, NewProject, ProjectName, UserId};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateProjectArgs {
    #[structopt(name = "project-name")]
    /// Full name of the new project
    name: ProjectName,

    #[structopt(long = "title")]
    /// Set the title of the new project
    title: Option<String>,

    #[structo