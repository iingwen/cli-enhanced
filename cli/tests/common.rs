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
    Lazy::new(|| env::var("REINFER_CLI_TEST_TOKEN").ok());

static TEST_CLI: Lazy<TestCli> = Lazy::new(|| {
    let cli_path = std::env::current_exe()
        .ok()
        .and_then(|p| Some(p.parent()?.parent()?.join("re")))
        .expect("Could not resolve CLI executable from test executable");

    TestCli { cli_path }
});

pub struct TestCli {
    cli_path: PathBuf,
}

impl TestCli {
    pub fn get() -> &'static Self {
        &TEST_CLI
    }

    pub fn user(&self) -> User {
        let output = self.run(["--output=json", "get", "current-user"]);
        serde_json::from_str::<User>(&output).expect("Failed to deserialize user response")
    }

    pub fn project() -> String {
        REINFER_CLI_TEST_PROJECT.to_owned()
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.cli_path);

        match (&*REINFER_CLI_TEST_CONTEXT, &*REINFER_CLI_TEST_ENDPOINT, &*REINFER_CLI_TEST_TOKEN) {
            (Some(context), _, _) => {
                command.arg("--context").arg(context);
            },
            (_, Some(endpoint), Some(token)) => {
                command
                    .arg("--endpoint")
                    .arg(endpoint)
                    .arg("--token")
                    .arg(token);
            },
            _ => panic!("Either REINFER_CLI_TEST_CONTEXT, or REINFER_CLI_TEST_ENDPOINT and REINFER_CLI_TEST_TOKEN must be set.")
        }

      