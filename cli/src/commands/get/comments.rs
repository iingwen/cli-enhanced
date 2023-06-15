
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
    /// Filter to comments only from these senders
    senders: Option<Vec<String>>,

    #[structopt(long = "recipients")]
    /// Filter to emails only to these recipients
    recipients: Option<Vec<String>>,

    #[structopt(short = "f", long = "file", parse(from_os_str))]
    /// Path where to write comments as JSON. If not specified, stdout will be used.
    path: Option<PathBuf>,

    #[structopt(short = "l", long = "label-filter")]
    /// Regex filter to select which labels you want to download predictions for
    label_filter: Option<Regex>,

    #[structopt(short = "p", long = "user-property-filter")]
    /// The user property filter to use as a json string
    property_filter: Option<StructExt<UserPropertiesFilter>>,

    #[structopt(long = "interactive-user-property-filter")]
    /// Open a dialog to interactively construct the user property filter to use
    interactive_property_filter: bool,

    #[structopt(long = "attachment-types")]
    /// The list of attachment types to filter to
    attachment_type_filters: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StructExt<T>(pub T);

impl<T: serde::de::DeserializeOwned> FromStr for StructExt<T> {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        serde_json::from_str(string).map_err(|source| {
            anyhow!(
                "Expected valid json for type. Got: '{}', which failed because: '{}'",
                string.to_owned(),
                source
            )
        })
    }
}

pub fn get_single(client: &Client, args: &GetSingleCommentArgs) -> Result<()> {
    let GetSingleCommentArgs {
        source,
        comment_id,
        path,
    } = args;
    let file: Option<Box<dyn Write>> = match path {
        Some(path) => Some(Box::new(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        )),
        None => None,
    };

    let stdout = io::stdout();
    let mut writer: Box<dyn Write> = file.unwrap_or_else(|| Box::new(stdout.lock()));
    let source = client
        .get_source(source.to_owned())
        .context("Operation to get source has failed.")?;
    let comment = client.get_comment(&source.full_name(), comment_id)?;
    print_resources_as_json(
        std::iter::once(AnnotatedComment {
            comment,
            labelling: None,
            entities: None,
            thread_properties: None,
            moon_forms: None,
            label_properties: None,
        }),
        &mut writer,
    )
}

fn get_user_properties_filter_interactively(
    client: &Client,
    dataset_id: DatasetIdentifier,
) -> Result<UserPropertiesFilter> {
    let dataset = client.get_dataset(dataset_id)?;

    let dataset_summary = client.dataset_summary(
        &dataset.full_name(),
        &SummaryRequestParams {
            attribute_filters: Vec::new(),
            filter: CommentFilter::default(),
        },
    )?;

    let mut properties: Vec<String> = dataset_summary
        .summary
        .user_properties
        .string
        .iter()
        .map(|p| p.full_name.clone())
        .collect();

    let mut filters = HashMap::new();

    loop {
        if properties.is_empty() {
            info!("No user properties available!");
            break;
        }

        let selected_property = Select::new()
            .with_prompt("Which user property would you like to filter?")
            .items(&properties)
            .interact()?;

        let selected_property_name = properties.remove(selected_property);

        let mut values: Vec<PropertyValue> = Vec::new();
        loop {
            let value: String = Input::new()
                .with_prompt("What value would you like to filter on?")
                .interact()?;
            values.push(PropertyValue::String(value));

            if !Confirm::new()
                .with_prompt(format!(
                    "Do you want to filter '{selected_property_name}' on more values?",
                ))
                .interact()?
            {
                break;
            }
        }

        let property_filter = PropertyFilter::new(values, Vec::new(), Vec::new());

        filters.insert(selected_property_name, property_filter);

        if !Confirm::new()
            .with_prompt("Do you want to filter additional user properties?")
            .interact()?
        {
            break;
        }
    }

    Ok(UserPropertiesFilter(filters))
}

pub fn get_many(client: &Client, args: &GetManyCommentsArgs) -> Result<()> {
    let GetManyCommentsArgs {
        source,
        dataset,
        no_progress,
        include_predictions,
        model_version,
        reviewed_only,
        from_timestamp,
        to_timestamp,
        path,
        label_filter,
        attachment_type_filters,
        property_filter: user_property_filter,
        interactive_property_filter: interative_property_filter,
        recipients,
        senders,
    } = args;

    let by_timerange = from_timestamp.is_some() || to_timestamp.is_some();
    if reviewed_only.unwrap_or_default() && by_timerange {
        bail!("The `reviewed_only` and `from/to-timestamp` options are mutually exclusive.")
    }

    let reviewed_only = reviewed_only.unwrap_or(false);
    if reviewed_only && model_version.is_some() {
        bail!("The `reviewed_only` and `model_version` options are mutually exclusive.")
    }

    if reviewed_only && dataset.is_none() {
        bail!("Cannot get reviewed comments when `dataset` is not provided.")
    }

    if include_predictions.unwrap_or_default() && dataset.is_none() {
        bail!("Cannot get predictions when `dataset` is not provided.")
    }

    if label_filter.is_some() && dataset.is_none() {
        bail!("Cannot use a label filter when `dataset` is not provided.")
    }

    if !attachment_type_filters.is_empty() && dataset.is_none() {
        bail!("Cannot use a attachment type filter when `dataset` is not provided.")
    }

    if label_filter.is_some() && reviewed_only {
        bail!("The `reviewed_only` and `label_filter` options are mutually exclusive.")
    }

    if label_filter.is_some() && model_version.is_some() {
        bail!("The `label_filter` and `model_version` options are mutually exclusive.")
    }

    if (user_property_filter.is_some() || *interative_property_filter) && dataset.is_none() {
        bail!("Cannot use a property filter when `dataset` is not provided.")
    }

    if (user_property_filter.is_some() || *interative_property_filter) && reviewed_only {
        bail!("The `reviewed_only` and `property_filter` options are mutually exclusive.")
    }

    if (user_property_filter.is_some() || *interative_property_filter)
        && user_property_filter.is_some()
    {
        bail!("The `interative_property_filter` and `property_filter` options are mutually exclusive.")
    }

    if (senders.is_some() || recipients.is_some()) && dataset.is_none() {
        bail!("Cannot filter on `senders` or `recipients` when `dataset` is not provided")
    }

    let file = match path {
        Some(path) => Some(
            File::create(path)
                .with_context(|| format!("Could not open file for writing `{}`", path.display()))
                .map(BufWriter::new)?,
        ),
        None => None,
    };

    let mut label_attribute_filter: Option<AttributeFilter> = None;
    if let (Some(dataset_id), Some(filter)) = (dataset, label_filter) {
        label_attribute_filter = get_label_attribute_filter(client, dataset_id.clone(), filter)?;
        // Exit early if no labels match label filter
        if label_attribute_filter.is_none() {
            return Ok(());
        }
    }

    let mut attachment_property_types_filter: Option<AttributeFilter> = None;

    if !attachment_type_filters.is_empty() {
        attachment_property_types_filter = Some(AttributeFilter {
            attribute: Attribute::AttachmentPropertyTypes,
            filter: AttributeFilterEnum::StringAnyOf {
                any_of: attachment_type_filters.to_vec(),
            },
        });
    }

    let user_properties_filter = if let Some(filter) = user_property_filter {
        Some(filter.0.clone())
    } else if *interative_property_filter {
        Some(get_user_properties_filter_interactively(
            client,
            dataset.clone().context("Could not unwrap dataset")?,
        )?)
    } else {
        None
    };

    let messages_filter = MessagesFilter {
        from: senders.as_ref().map(|senders| {
            PropertyFilter::new(
                senders
                    .iter()
                    .map(|sender| PropertyValue::String(sender.to_owned()))
                    .collect(),
                Vec::new(),
                Vec::new(),
            )
        }),
        to: recipients.as_ref().map(|recipients| {
            PropertyFilter::new(
                recipients
                    .iter()
                    .map(|recipient| PropertyValue::String(recipient.to_owned()))
                    .collect(),
                Vec::new(),
                Vec::new(),
            )