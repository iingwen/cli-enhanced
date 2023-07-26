use crate::{
    commands::DEFAULT_TRANSFORM_TAG,
    parse::{get_files_in_directory, Statistics},
};
use anyhow::{anyhow, Context, Result};
use cfb::Compound