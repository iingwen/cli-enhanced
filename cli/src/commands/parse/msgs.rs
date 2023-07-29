use crate::{
    commands::DEFAULT_TRANSFORM_TAG,
    parse::{get_files_in_directory, Statistics},
};
use anyhow::{anyhow, Context, Result};
use cfb::CompoundFile;
use colored::Colorize;
use log::error;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{io::Read, sync::Arc};

use reinfer_client::{
    resources::{
        documents::{Document, RawEmail, RawEmailBody, RawEmailHeaders},
        email::AttachmentMetadata,
    },
    Client, PropertyMap, SourceIdentifier, TransformTag,
};
use std::{
    fs::File,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

use crate::{
    commands::ensure_uip_user_consents_to_ai_unit_charge,
    progress::{Options as ProgressOptions, Progress},
};

use super::upload_batch_of_documents;

const MSG_NAME_USER_PROPERTY_NAME: &str = "MSG NAME ID";
const STREAM_PATH_ATTACHMENT_STORE_PREFIX: &str = "__attach_version1.0_#";
const UPLOAD_BATCH_SIZE: usize = 128;

static CONTENT_TYPE_MIME_HEADER_RX: Lazy<Regex> =
    Lazy::new(|| Regex::new