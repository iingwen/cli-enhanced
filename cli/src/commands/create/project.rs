use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{Client, NewProject, ProjectName, UserId};
use structopt::StructOpt;

