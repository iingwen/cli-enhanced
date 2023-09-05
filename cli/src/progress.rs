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
            Arc::clone(&report_progress_flag),
        );

        Progress {
            report_progress_flag,
            progress_thread: Some(progress_thread),
        }
    }

    pub fn done(&mut self) {
        if let Some(handle) = self.progress_thread.take() {
            self.report_progress_flag.store(false, Ordering::SeqCst);
            handle.join().expect("Could not join progress thread.");
        }
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.done();
    }
}

fn spawn_progress_thread<Statistics, ProgressFn>(
    statistics: Arc<Statistics>,
    progress_fn: ProgressFn,
    max_progress_value: Option<u64>,
    options: Options,
    report_progress: Arc<AtomicBool>,
) -> thread::JoinHandle<()>
where
    ProgressFn: Fn(&Statistics) -> ProgressMessage + Sync + Send + 'static,
    Statistics: Sync + Send + 'static,
{
    use std::ops::Deref;
    let mut template_str = String::new();
    write!(template_str, "{} ", LOG_PREFIX_INFO.deref()).unwrap();
    write!(template_str, "{{spinner:.green}} ").unwrap();
    write!(template_str, "[{{elapsed_precise}}] {{prefix}} ").unwrap();

    match (max_progress_value.is_some(), options.bytes_units) {
        (true, true) => write!(
            template_str,
            "{{bar:32.cyan/blue}} {{bytes}} / {{total_bytes}} ({{eta}})"
        )
        .unwrap(),
        (true, false) => write!(template_str, "{{bar:32.cyan/blue}}