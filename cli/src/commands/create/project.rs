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

    #[structopt(long = "description")]
    /// Set the description of the new project
    description: Option<String>,

    #[structopt(long = "user-ids", required = true)]
    /// The ids of users to be given initial control of the new project
    user_ids: Vec<UserId>,
}

pub fn create(cl