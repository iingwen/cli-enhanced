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
p