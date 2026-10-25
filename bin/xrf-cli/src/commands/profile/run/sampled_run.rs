use std::path::Path;
use std::process::{Child, Command as ChildCommand, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use xrf_error::XrfError;
use xrf_utils::format_path;

use crate::commands::profile::run::resident_samples::ResidentSamples;
use crate::core::command::CommandResult;

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

impl SampledRun {
  /// Runs one invocation, sampling its resident set on a second thread while it lives.
  ///
  /// The wait is blocking on this thread rather than a poll loop, so the duration is the real one: a loop that slept
  /// between checks would quantize every measurement to the sampling interval, which on a fast command is most of the
  /// figure. Sampling needs only the process id, so it costs the timing nothing.
  ///
  /// # Errors
  ///
  /// Returns an error when the binary cannot be started, or when waiting on it fails.
  pub fn measure(path: &Path, arguments: &[String]) -> CommandResult<Self> {
    // Discarded rather than inherited or piped: inherited streams put the child's logging into this run's, and a pipe
    // nobody drains can fill its buffer and block the very work being timed.
    let mut child: Child = ChildCommand::new(path)
      .args(arguments)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|error| {
        XrfError::new_io_error(format!("Failed to run '{}': {error}", format_path(path)), error.kind())
      })?;

    let samples: Arc<ResidentSamples> = Arc::new(ResidentSamples::default());
    let is_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let sampler: JoinHandle<()> = ResidentSamples::spawn_sampler(&samples, child.id(), Arc::clone(&is_running));

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

    Ok(Self {
      elapsed,
      exit_code: status.code().unwrap_or(-1),
      peak_bytes,
      mean_bytes,
    })
  }
}
