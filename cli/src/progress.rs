use indicatif::{ProgressBar, ProgressStyle};
use std::fmt::Write;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::utils::LOG_PREFIX_INFO;

pub type ProgressMessage = (u64, String);

pub struct 