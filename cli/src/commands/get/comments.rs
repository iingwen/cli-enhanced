
use anyhow::{anyhow, bail, Context, Error, Result};

use chrono::{DateTime, Utc};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};
use log::info;
use regex::Regex;
use reinfer_client::{
    resources::{
        comment::{
            CommentTimestampFilter, MessagesFilter, PredictedLabelName, PropertyFilter,
            ReviewedFilterEnum, UserPropertiesFilter,
        },
        dataset::{
            Attribute, AttributeFilter, AttributeFilterEnum, OrderEnum, QueryRequestParams,
            StatisticsRequestParams as DatasetStatisticsRequestParams, SummaryRequestParams,
        },
        source::StatisticsRequestParams as SourceStatisticsRequestParams,
    },
    AnnotatedComment, Client, CommentFilter, CommentId, CommentsIterTimerange, DatasetFullName,
    DatasetIdentifier, Entities, HasAnnotations, LabelName, Labelling, ModelVersion,
    PredictedLabel, PropertyValue, Source, SourceIdentifier, DEFAULT_LABEL_GROUP_NAME,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use structopt::StructOpt;

use crate::{
    printer::print_resources_as_json,
    progress::{Options as ProgressOptions, Progress},
};

#[derive(Debug, StructOpt)]
pub struct GetSingleCommentArgs {
    #[structopt(long = "source")]
    /// Source name or id
    source: SourceIdentifier,

    #[structopt(name = "comment-id")]
    /// Comment id.
    comment_id: CommentId,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write comments as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct GetManyCommentsArgs {
    #[structopt(name = "source")]
    /// Source name or id
    source: SourceIdentifier,

    #[structopt(short = "d", long = "dataset")]
    /// Dataset name or id
    dataset: Option<DatasetIdentifier>,

    #[structopt(long)]
    /// Don't display a progress bar (only applicable when --file is used).
    no_progress: bool,

    #[structopt(long = "predictions")]
    /// Save predicted labels and entities for each comment.
    include_predictions: Option<bool>,

    #[structopt(long = "model-version")]
    /// Get predicted labels and entities from the specified model version rather than latest.
    model_version: Option<u32>,

    #[structopt(long = "reviewed-only")]
    /// Download reviewed comments only.
    reviewed_only: Option<bool>,

    #[structopt(long = "from-timestamp")]
    /// Starting timestamp for comments to retrieve (inclusive).
    from_timestamp: Option<DateTime<Utc>>,

    #[structopt(long = "to-timestamp")]
    /// Ending timestamp for comments to retrieve (inclusive).
    to_timestamp: Option<DateTime<Utc>>,

    #[structopt(long = "senders")]