use anyhow::{Context, Result};

use colored::Colorize;
use reinfer_client::{BucketIdentifier, Client};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    sync::{
        atomic