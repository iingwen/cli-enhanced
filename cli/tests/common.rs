use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use reinfer_client::User;
use std::{
    env,
    f