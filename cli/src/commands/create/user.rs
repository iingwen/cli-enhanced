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

    #[structopt(name = "e