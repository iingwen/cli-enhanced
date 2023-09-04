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
    pub fn new<ProgressFnT, StatisticsT>(
        progress_fn: ProgressFnT,
        statistics: &Arc<StatisticsT>,
        target_value: Option<u64>,
        options: Options,
    ) -> Self
    where
        ProgressFnT: Fn(&StatisticsT) -> ProgressMessage + Sync + Send + 'static,
        StatisticsT: Sync + Send + 'static,
    {
        let report_progress_flag = Arc::new(AtomicBool::new(true));
        let progress_thread = spawn_progress_thread(
            Arc::clone(statistics),
            progress_fn,
            target_value,
            options,
            Ar