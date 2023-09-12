use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use reinfer_client::User;
use std::{
    env,
    ffi::OsStr,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

static REINFER_CLI_TEST_PROJECT: Lazy<String> = Lazy::new(|| {
    env::var("REINFER_CLI_TEST_PROJECT")
        .expect("REINFER_CLI_TEST_PROJECT must be set for integration tests")
});
static REINFER_CLI_TEST_ENDPOINT: Lazy<Option<String>> =
    Lazy::new(|| env::var("REINFER_CLI_TEST_ENDPOINT").ok());
static REINFER_CLI_TEST_CONTEXT: Lazy<Option<String>> =
    Lazy::new(|| env::var("REINFER_CLI_TEST_CONTEXT").ok());
static REINFER_CLI_TEST_TOKEN: Lazy<Option<String>> =
    Lazy::new(|| 