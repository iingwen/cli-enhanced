use crate::printer::Printer;
use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    Client, CommentUid, StreamException, StreamExceptionMetadata, StreamFullName,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct CreateStreamExceptionArgs {
    #[structopt(long = "stream")]
    /// The stream full name, qualified by da