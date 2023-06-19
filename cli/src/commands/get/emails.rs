use anyhow::{Context, Result};

use colored::Colorize;
use reinfer_client::{BucketIdentifier, Client};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::{
    printer::print_resources_as_json,
    progress::{Options as ProgressOptions, Progress},
};

#[derive(Debug, StructOpt)]
pub struct GetManyEmailsArgs {
    #