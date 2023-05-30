use anyhow::{Context, Result};
use reinfer_client::{
    Client, Email, GlobalPermission, NewUser, ProjectName, ProjectPermission, Username,
};
use std::collections::hash_map::HashMap;
use structopt::StructOpt;

use crate::printer::Printer;

#[derive(Debug, StructOpt)]
pub struct CreateUserArgs {
    #[structopt(name = "username")]
    /// Username for the new user
    username: Username,

    #[structopt(name = "email")]
    /// Email address of the new user
    email: Email,

    #[structopt(long = "global-permissions")]
    /// Global permissions to give to the new user
    global_permissions: Vec<GlobalPermission>,

    #[structopt(short = "p", long = "project")]
    /// Add the user to this project with the permissions provided with --project-permissions
    project: Option<ProjectName>,

    #[structopt(long = "project-permissions")]
    /// Project permissions, required if --project is used
    project_permissions_list: Vec<ProjectPermission>,

    #[structopt(short = "w", long = "send-welcome-email")]
    /// Send the user a welcome email
    send_welcome_email: bool,
}

pub fn create(client: &Client, args: &CreateUserArgs, printer: &Printer) -> Result<()> {
    let CreateUserArgs {
        username,
        email,
        global_permissions,
        project,
        project_permissions_list,
        send_welcome_email,
    } = args;

    let project_permissions = match (project, project_permissions_list) {
        (Some(project), permissions) if !permissions.is_empty() => maplit::hashmap!(
            project.clone() => permissions.iter().cloned().collect()
        ),
        (None, permissions) if permissions.is_empty() => HashMap::new(),
        _ => {
            anyhow::bail!(
                "Arguments `--project` and `--project-permissions` have to be both specified or neither"
            );
      