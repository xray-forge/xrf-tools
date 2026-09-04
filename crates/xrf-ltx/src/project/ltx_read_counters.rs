use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// How much reading and parsing a project actually did.
#[derive(Debug, Default)]
pub(crate) struct LtxReadCounters {
  bytes_read: AtomicU64,
  reads: AtomicU64,
  parses: AtomicU64,
  include_scans: AtomicU64,
  resolutions: AtomicU64,
}

impl LtxReadCounters {
  /// A shared handle, which is how a project hands its counters to the sources it builds.
  pub(crate) fn new_shared() -> Arc<Self> {
    Arc::new(Self::default())
  }

  /// Records one config read out of the mounted sources.
  pub(crate) fn record_read(&self, bytes: u64) {
    self.reads.fetch_add(1, Ordering::Relaxed);
    self.bytes_read.fetch_add(bytes, Ordering::Relaxed);
  }

  /// Records one full parse of a config into sections.
  pub(crate) fn record_parse(&self) {
    self.parses.fetch_add(1, Ordering::Relaxed);
  }

  /// Records one include-only parse, which reads a config's `#include` statements and discards its contents.
  pub(crate) fn record_include_scan(&self) {
    self.include_scans.fetch_add(1, Ordering::Relaxed);
  }

  /// Records one root resolved with its includes merged and its inheritance flattened.
  pub(crate) fn record_resolution(&self) {
    self.resolutions.fetch_add(1, Ordering::Relaxed);
  }

  /// A snapshot of the counts so far.
  pub(crate) fn get_snapshot(&self) -> LtxReadCountersSnapshot {
    LtxReadCountersSnapshot {
      bytes_read: self.bytes_read.load(Ordering::Relaxed),
      include_scans: self.include_scans.load(Ordering::Relaxed),
      parses: self.parses.load(Ordering::Relaxed),
      reads: self.reads.load(Ordering::Relaxed),
      resolutions: self.resolutions.load(Ordering::Relaxed),
    }
  }
}

/// The counts at one moment, detached from the project that produced them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxReadCountersSnapshot {
  /// Configs read out of the mounted sources, counting a file once per read rather than once per distinct path.
  pub reads: u64,
  /// Source bytes those reads returned.
  pub bytes_read: u64,
  /// Full parses of a config into sections.
  pub parses: u64,
  /// Include-only parses, which project assembly performs over every config before any content is parsed.
  pub include_scans: u64,
  /// Roots resolved with includes merged and inheritance flattened.
  pub resolutions: u64,
}
