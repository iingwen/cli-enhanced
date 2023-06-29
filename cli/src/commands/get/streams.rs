use anyhow::{anyhow, Context, Result};
use colored::{ColoredString, Colorize};
use log::info;
use ordered_float::NotNan;
use prettytable::row;
use reinfer_client::resources::stream::{StreamLabelThreshold, StreamModel};
use reinfer_client::re