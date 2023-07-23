use anyhow::{bail, Context, Result};
use reinfer_client::{Client, ProjectName, ProjectPermission, UserIdentifier};
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct GetUsersArgs {
    #[structopt(short = "u", long = "user")]
    /// Use to retrieve a single user with the provided id
    user: Option<UserIdentifier>,

    #[structopt(short = "o", long = "project")]
    /// Filter users by a given project
    project_name_filter: Option<ProjectName>,

    #[structopt(short = "p", long = "permission")]
    /// Filter users by a given project permission
    project_permission_filter: Option<ProjectPermission>,
}

pub fn get(client: &Client, args: &GetUsersArgs, printe