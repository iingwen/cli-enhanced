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

pub struct Options {
    pub bytes_units: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { bytes_units: true }
    }
}

pub struct Progress {
    report_progress_flag: Arc<AtomicBool>,
    progress_thread: Option<thread::JoinHandle<()>>,
}

impl Progress {
    pub fn n