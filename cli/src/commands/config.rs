
use colored::Colorize;
use log::{error, info, warn};
use prettytable::{self, row, Table};
use reinfer_client::DEFAULT_ENDPOINT;
use reqwest::Url;
use std::path::Path;
use structopt::StructOpt;

use crate::{
    config::{self, write_reinfer_config, ContextConfig, ReinferConfig},
    utils,
};
use anyhow::{anyhow, Result};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, StructOpt)]
pub enum ConfigArgs {
    #[structopt(name = "add")]
    /// Add a new context to the reinfer config file
    AddContext {
        #[structopt(long = "name", short = "n")]
        /// The name of the context that will be created or updated
        name: Option<String>,

        #[structopt(long = "endpoint", short = "e")]
        /// The reinfer cluster endpoint that will be used for this context
        endpoint: Option<Url>,

        #[structopt(long = "token", short = "t")]
        /// The reinfer API token that will be used for this context
        token: Option<String>,

        #[structopt(long = "accept-invalid-certificates", short = "k")]
        /// Whether to accept invalid TLS certificates
        accept_invalid_certificates: bool,

        #[structopt(long = "proxy")]
        /// URL for an HTTP proxy that will be used for all requests if specified
        proxy: Option<Option<Url>>,
    },

    /// Output the token for a given context or the current one if unspecified.
    #[structopt(name = "get-token")]
    GetToken { name: Option<String> },

    #[structopt(name = "current")]
    /// Display the current context
    CurrentContext,

    #[structopt(name = "delete")]
    /// Delete the specified context from the reinfer config file
    DeleteContext {
        /// The name(s) of the context(s) which will be deleted
        names: Vec<String>,
    },

    #[structopt(name = "ls")]
    /// List available contexts in a reinfer config file
    ListContexts {
        #[structopt(long = "tokens")]
        /// Show API tokens (by default tokens are hidden).
        tokens: bool,
    },

    #[structopt(name = "use")]
    /// Set the current context in the reinfer config file
    UseContext {
        /// The name of the context.
        name: String,
    },

    #[structopt(name = "set-context-required")]
    /// Set whether context is a required field
    SetContextRequired {
        // Whether the context is a required field
        #[structopt(name = "is-required", parse(try_from_str))]
        is_required: bool,