mod dataset;
mod project;
mod source;
mod users;

use self::{
    dataset::UpdateDatasetArgs, project::UpdateProjectArgs, source::UpdateSourceArgs,
    users::UpdateUsersArgs,
};
use crate::printer::Printer;
use anyhow::Result;
use reinfer_client::Client;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum UpdateArgs {
    #[structopt(name = "source")]
    /// Update an existing source
    Source(UpdateSourceArgs),

    #[structopt(name = "dataset")]
    /// Update an existing dataset
    Dataset(UpdateDatasetArgs),

    #[structopt(name = "project")]
    /// Update an e