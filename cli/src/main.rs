
#![deny(clippy::all)]
mod args;
mod commands;
mod config;
mod printer;
mod progress;
mod thousands;
mod utils;

use anyhow::{anyhow, Context, Result};
use log::{error, warn};
use reinfer_client::{
    retry::{RetryConfig, RetryStrategy},
    Client, Config as ClientConfig, Token, DEFAULT_ENDPOINT,
};
use scoped_threadpool::Pool;
use std::{env, fs, io, path::PathBuf, process};
use structopt::{clap::Shell as ClapShell, StructOpt};

use crate::{
    args::{Args, Command, Shell},
    commands::{config as config_command, create, delete, get, parse, update},
    config::ReinferConfig,
    printer::Printer,
};

const NUM_THREADS_ENV_VARIABLE_NAME: &str = "REINFER_CLI_NUM_THREADS";

fn run(args: Args) -> Result<()> {
    let config_path = find_configuration(&args)?;
    let config = config::read_reinfer_config(&config_path)?;
    let printer = Printer::new(args.output);

    let number_of_threads = if let Ok(num_threads_env_var_str) =
        env::var(NUM_THREADS_ENV_VARIABLE_NAME)
    {
        num_threads_env_var_str
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("Environment variable {NUM_THREADS_ENV_VARIABLE_NAME} is not a u32: '{num_threads_env_var_str}'"))
    } else {
        args.num_threads
    };

    let mut pool = Pool::new(number_of_threads);

    match &args.command {
        Command::Config { config_args } => {
            config_command::run(config_args, config, config_path).map(|_| ())
        }
        Command::Completion { shell } => {
            let mut app = Args::clap();
            let clap_shell = match shell {
                Shell::Zsh => ClapShell::Zsh,
                Shell::Bash => ClapShell::Bash,
            };
            app.gen_completions_to("re", clap_shell, &mut io::stdout());
            Ok(())
        }
        Command::Get { get_args } => get::run(
            get_args,
            client_from_args(&args, &config)?,
            &printer,
            &mut pool,