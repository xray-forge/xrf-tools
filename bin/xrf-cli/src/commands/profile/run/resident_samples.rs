use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// How often a running child's resident set is read.
///
/// Twenty milliseconds is fine enough to catch a peak that lasts a fraction of a second, and coarse enough that the
/// sampler is not competing with the workload it is measuring. Polling without a sleep steals a core outright and
/// inflates every number it produces. It is also what makes the mean meaningful: evenly spaced samples average to the
/// time the process spent at a size, where samples taken whenever the sampler happened to run would not.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(20);

/// Running totals a sampler thread writes and its owner reads once the child is reaped.
#[derive(Debug, Default)]
pub struct ResidentSamples {
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
  pub fn summarize(&self) -> (Option<u64>, Option<u64>) {
    match self.count.load(Ordering::Relaxed) {
      0 => (None, None),
      count => (
        Some(self.peak.load(Ordering::Relaxed)),
        Some(self.total.load(Ordering::Relaxed) / count),
      ),
    }
  }

  /// Watches one process id until told to stop, recording every resident set it saw into `samples`.
  ///
  /// Stops on the flag rather than on the process disappearing, so the window in which the operating system could hand
  /// this id to something else is one interval at most — and the flag is set the instant the child is reaped.
  pub fn spawn_sampler(samples: &Arc<Self>, pid: u32, is_running: Arc<AtomicBool>) -> JoinHandle<()> {
    let samples: Arc<Self> = Arc::clone(samples);

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
