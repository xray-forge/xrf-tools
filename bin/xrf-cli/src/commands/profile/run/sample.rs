use std::path::Path;
use std::process::{Child, Command as ChildCommand, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use xrf_error::XrfError;
use xrf_utils::format_path;

use crate::core::generic_command::CommandResult;

/// How often a running child's resident set is read.
///
/// Twenty milliseconds is fine enough to catch a peak that lasts a fraction of a second, and coarse enough that the
/// sampler is not competing with the workload it is measuring. Polling without a sleep steals a core outright and
/// inflates every number it produces. It is also what makes the mean meaningful: evenly spaced samples average to the
/// time the process spent at a size, where samples taken whenever the sampler happened to run would not.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(20);

/// What one measured invocation cost.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampledRun {
  pub elapsed: Duration,
  pub exit_code: i32,
  /// Largest resident set seen while the process lived, or `None` when it ended before a sample landed.
  pub peak_bytes: Option<u64>,
  /// Mean resident set across the samples, which is what the process cost the machine for most of its life.
  pub mean_bytes: Option<u64>,
}

/// Running totals a sampler thread writes and its owner reads once the child is reaped.
#[derive(Debug, Default)]
struct ResidentSamples {
  peak: AtomicU64,
  total: AtomicU64,
  count: AtomicU64,
}

impl ResidentSamples {
  fn record(&self, bytes: u64) {
    self.peak.fetch_max(bytes, Ordering::Relaxed);
    self.total.fetch_add(bytes, Ordering::Relaxed);
    self.count.fetch_add(1, Ordering::Relaxed);
  }

  /// The peak and the mean, or `None` for each when nothing was sampled.
  fn summarize(&self) -> (Option<u64>, Option<u64>) {
    match self.count.load(Ordering::Relaxed) {
      0 => (None, None),
      count => (
        Some(self.peak.load(Ordering::Relaxed)),
        Some(self.total.load(Ordering::Relaxed) / count),
      ),
    }
  }
}

/// Runs one invocation, sampling its resident set on a second thread while it lives.
///
/// The wait is blocking on this thread rather than a poll loop, so the duration is the real one: a loop that slept
/// between checks would quantize every measurement to the sampling interval, which on a fast command is most of the
/// figure. Sampling needs only the process id, so it costs the timing nothing.
///
/// # Errors
///
/// Returns an error when the binary cannot be started, or when waiting on it fails.
pub fn run_sampled(path: &Path, arguments: &[String]) -> CommandResult<SampledRun> {
  // Discarded rather than inherited or piped: inherited streams put the child's logging into this run's, and a pipe
  // nobody drains can fill its buffer and block the very work being timed.
  let mut child: Child = ChildCommand::new(path)
    .args(arguments)
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .map_err(|error| XrfError::new_io_error(format!("Failed to run '{}': {error}", format_path(path)), error.kind()))?;

  let samples: Arc<ResidentSamples> = Arc::new(ResidentSamples::default());
  let is_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
  let sampler: JoinHandle<()> = spawn_sampler(child.id(), Arc::clone(&samples), Arc::clone(&is_running));

  let started_at: Instant = Instant::now();
  let status = child.wait();
  let elapsed: Duration = started_at.elapsed();

  is_running.store(false, Ordering::Relaxed);

  // Joined before reading the totals, so the last sample is included rather than raced against.
  let _ = sampler.join();

  let status = status.map_err(|error| {
    XrfError::new_io_error(
      format!("Failed to wait on '{}': {error}", format_path(path)),
      error.kind(),
    )
  })?;

  let (peak_bytes, mean_bytes) = samples.summarize();

  Ok(SampledRun {
    elapsed,
    exit_code: status.code().unwrap_or(-1),
    peak_bytes,
    mean_bytes,
  })
}

/// Watches one process id until told to stop, recording every resident set it saw.
///
/// Stops on the flag rather than on the process disappearing, so the window in which the operating system could hand
/// this id to something else is one interval at most — and the flag is set the instant the child is reaped.
fn spawn_sampler(pid: u32, samples: Arc<ResidentSamples>, is_running: Arc<AtomicBool>) -> JoinHandle<()> {
  thread::spawn(move || {
    let pid: Pid = Pid::from_u32(pid);
    let refresh: ProcessRefreshKind = ProcessRefreshKind::nothing().with_memory();
    let mut system: System = System::new();

    while is_running.load(Ordering::Relaxed) {
      system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, refresh);

      if let Some(process) = system.process(pid) {
        samples.record(process.memory());
      }

      thread::sleep(SAMPLE_INTERVAL);
    }
  })
}

#[cfg(test)]
mod tests {
  use super::ResidentSamples;

  #[test]
  fn summarizes_nothing_before_a_sample_lands() {
    assert_eq!(ResidentSamples::default().summarize(), (None, None));
  }

  /// The peak is the largest sample and the mean is what the process held for most of its life; a brief spike moves one
  /// and barely moves the other, which is the whole reason both are reported.
  #[test]
  fn separates_a_brief_spike_from_a_retained_allocation() {
    let samples: ResidentSamples = ResidentSamples::default();

    for bytes in [100, 100, 100, 900, 100, 100, 100, 100] {
      samples.record(bytes);
    }

    assert_eq!(samples.summarize(), (Some(900), Some(200)));
  }

  #[test]
  fn summarizes_a_steady_process_as_one_figure_twice() {
    let samples: ResidentSamples = ResidentSamples::default();

    samples.record(512);
    samples.record(512);

    assert_eq!(samples.summarize(), (Some(512), Some(512)));
  }
}
