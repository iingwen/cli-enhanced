
use super::thousands::Thousands;
use colored::Colorize;
use prettytable::{format, row, Row, Table};
use reinfer_client::{
    resources::{
        audit::PrintableAuditEvent, bucket_statistics::Statistics as BucketStatistics,
        dataset::DatasetAndStats, integration::Integration, quota::Quota,
    },
    Bucket, CommentStatistics, Dataset, Project, Source, Stream, User,
};
use serde::{Serialize, Serializer};

use anyhow::{anyhow, Context, Error, Result};
use std::{
    io::{self, Write},
    str::FromStr,
};

pub fn print_resources_as_json<Resource>(
    resources: impl IntoIterator<Item = Resource>,
    mut writer: impl Write,
) -> Result<()>
where
    Resource: Serialize,
{
    for resource in resources {
        serde_json::to_writer(&mut writer, &resource)
            .context("Could not serialise resource.")
            .and_then(|_| writeln!(writer).context("Failed to write JSON resource to writer."))?;
    }
    Ok(())
}

#[derive(Copy, Clone, Default, Debug)]
pub enum OutputFormat {
    Json,
    #[default]
    Table,
}

impl FromStr for OutputFormat {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        if string == "table" {
            Ok(OutputFormat::Table)
        } else if string == "json" {
            Ok(OutputFormat::Json)
        } else {
            Err(anyhow!("{}", string))
        }
    }
}

/// Represents a resource that is able to be displayed as a table.
///
/// The implementation must implement `to_table_headers` to return headers for the resource type,
/// and `to_table_row`, which should return a data row for the given resource instance.
pub trait DisplayTable {
    fn to_table_headers() -> Row;

    fn to_table_row(&self) -> Row;
}

impl DisplayTable for Integration {
    fn to_table_headers() -> Row {
        row![bFg => "Project", "Name", "ID", "Created (UTC)", "Mailbox Count"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.owner.0,
            self.name.0,
            self.id.0,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            self.configuration.mailboxes.len()
        ]
    }
}
impl DisplayTable for Bucket {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Created (UTC)"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.0.dimmed(), "/".dimmed(), self.name.0);
        row![
            full_name,
            self.id.0,
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
        ]
    }
}

impl DisplayTable for Quota {
    fn to_table_headers() -> Row {
        row![bFg => "Kind", "Hard Limit", "Usage (Total)", "Usage %"]
    }

    fn to_table_row(&self) -> Row {
        row![
            self.quota_kind,
            Thousands(self.hard_limit),
            Thousands(self.current_max_usage),
            if self.hard_limit > 0 {
                format!(
                    "{:.0}%",
                    (self.current_max_usage as f64 / self.hard_limit as f64) * 100.0
                )
            } else {
                "N/A".dimmed().to_string()
            }
        ]
    }
}

impl DisplayTable for Dataset {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Title"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!("{}{}{}", self.owner.0.dimmed(), "/".dimmed(), self.name.0);
        row![
            full_name,
            self.id.0,
            self.updated_at.format("%Y-%m-%d %H:%M:%S"),
            self.title,
        ]
    }
}

impl DisplayTable for DatasetAndStats {
    fn to_table_headers() -> Row {
        row![bFg => "Name", "ID", "Updated (UTC)", "Title","Total Verbatims", "Num Reviewed"]
    }

    fn to_table_row(&self) -> Row {
        let full_name = format!(
            "{}{}{}",
            self.dataset.owner.0.dimmed(),
            "/".dimmed(),
            self.dataset.name.0