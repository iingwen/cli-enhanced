use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use structopt::StructOpt;

use reinfer_client::{
    resources::project::Fo