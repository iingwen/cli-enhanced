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
    Lazy::new(|| Regex::new(r"Content-Type:((\s)+.+\n)+").unwrap());
static CONTENT_TRANSFER_ENCODING_MIME_HEADER_RX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"Content-Transfer-Encoding:((\s)+.+\n)+").unwrap());
static STREAM_PATH_MESSAGE_BODY_PLAIN: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_1000001F"));
static STREAM_PATH_MESSAGE_HEADER: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_007d001F"));
static STREAM_PATH_ATTACHMENT_FILENAME: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_3707001F"));
static STREAM_PATH_ATTACHMENT_EXTENSION: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_3703001F"));
static STREAM_PATH_ATTACHMENT_DATA: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("__substg1.0_37010102"));

#[derive(Debug, StructOpt)]
pub struct ParseMsgArgs {
    #[structopt(short = "d", long = "dir", parse(from_os_str))]
    /// Directory containing the msgs
    directory: PathBuf,

    #[structopt(short = "s", long = "source")]
    /// Source name or id
    source: SourceIdentifier,

    #[structopt(long = "transform-tag")]
    /// Transform tag to use.
    transform_tag: Option<TransformTag>,

    #[structopt(short = "n", long = "no-charge")]
    /// Whether to attempt to bypass billing (internal only)
    no_charge: bool,

    #[structopt(short = "y", long = "yes")]
    /// Consent to ai unit charge. Suppresses confirmation prompt.
    yes: bool,
}

fn read_stream(stream_path: &Path, compound_file: &mut CompoundFile<File>) -> Result<Vec<u8>> {
    let data = {
        let mut stream = compound_file.open_stream(stream_path)?;
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        buffer
    };

    Ok(data)
}

fn read_unicode_stream_to_string(
    stream_path: &Path,
    compound_file: &mut CompoundFile<File>,
) -> Result<String> {
    if !compound_file.is_stream(stream_path) {
        return Err(anyhow!(
            "Could not find stream {}. Please check that you are using unicode msgs",
            stream_path.to_string_lossy()
        ));
    }

    // Stream data is a UTF16 string encoded as Vec[u8]
    let data = read_stream(stream_path, compound_file)?;
    Ok(utf16le_stream_to_string(&data))
}

// Decode a UTF-16LE data stream, given as raw bytes of Vec<u8>
fn utf16le_stream_to_string(data: &[u8]) -> String {
    let mut decoder = encoding_rs::UTF_16LE.new_decoder();

    // The amount of memory to reserve for writing at a time
    // We should only require one or two blocks for the vast majority of cases
    let block_length = data.len();
    let mut buffer: String = String::with_capacity(block_length);

    loop {
        let (coder_result, _, _) = decoder.decode_to_string(data, &mut buffer, true);
        use encoding_rs::CoderResult;
        match coder_result {
            // The output buffer was not big enough - increase and retry
            CoderResult::OutputFull => buffer.reserve(block_length),
            CoderResult::InputEmpty => return buffer,
        }
    }
}

fn get_attachment_store_path(attachment_number: usize) -> PathBuf {
    PathBuf::from(format!(
        "{}{:08}",
        STREAM_PATH_ATTACHMENT_STORE_PREFIX, attachment_number
    ))
}

fn read_attachment(
    attachment_path: PathBuf,
    compound_file: &mut CompoundFile<File>,
) -> Result<AttachmentMetadata> {
    let mut attachment_name_path = attachment_path.clone();
    attachment_name_path.push(&*STREAM_PATH_ATTACHMENT_FILENAME);

    let mut content_type_path = attachment_path.clone();
    content_type_path.push(&*STREAM_PATH_ATTACHMENT_EXTENSION);

    let mut data_path = attachment_path.clone();
    data_path.push(&*STREAM_PATH_ATTACHMENT_DATA);

    let name = read_unicode_stream_to_string(&attachment_name_path, compound_file)?;
    let content_type = read_unicode_stream_to_string(&content_type_path, compound_file)?;
    let data = read_stream(&data_path, compound_file)?;

    Ok(AttachmentMetadata {
        name,
        content_type,
        size: data.len() as u64,
    })
}

fn remove_content_headers(headers_string: String) -> Result<String> {
    let mut clean_headers_string: String;

    clean_headers_string = CONTENT_TYPE_MIME_HEADER_RX
        .replace(&headers_string, "")
        .to_string();

    clean_headers_string = CONTENT_TRANSFER_ENCODING_MIME_HEADER_RX
        .replace(&clean_headers_string, "")
        .to_string();

    Ok(clean_headers_string)
}

fn read_msg_to_document(path: &PathBuf) -> Result<Document> {
    if !path.is_file() {
        return Err(anyhow!("No such file: {:?}", path));
    }

    let mut compound_file = cfb::open(path)?;

    // Headers
    let headers_string =
        read_unicode_stream_to_string(&STREAM_PATH_MESSAGE_HEADER, &mut compound_file)?;

    // As the content type won't match the parsed value from the body in the msg
    let headers_string_no_content_headers = r