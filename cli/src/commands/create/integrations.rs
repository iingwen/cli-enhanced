use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use std::path::PathBuf;

use log::info;
use reinfer_client::{
    resources::integration::{Integration, NewIntegration},
    Client, IntegrationFullName,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateIntegrationArgs {
    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path to JSON file with integration
    path: PathBuf,

    #[structopt(name = "name")]
    /// Name of the new integration
    name: IntegrationFullName,

    #[structopt(long)]
    /// Whether to overwrite an existing integration with the same name
    overwrite: bool,
}

pub fn create(client: &Client, args: &CreateIntegrationArgs) -> Result<()> {
    let CreateIntegrationArgs {
        path,
        name,
        overwrite,
    } = args;

    let new_integration = read_integration(path)?;

    let mut integrations = client.get_integrations()?;

    integrations
        .retain(|integration| format!("{}/{}", integration.owner.0, integration.name.0) == name.0);

    if let Some(existing) = integrations.first() {
       