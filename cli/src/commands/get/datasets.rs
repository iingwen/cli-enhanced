use anyhow::{Context, Result};
use log::info;
use reinfer_client::{
    resources::dataset::{DatasetAndStats, DatasetStats, StatisticsRequestPar