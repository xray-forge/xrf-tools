use std::collections::HashMap;
use std::sync::Mutex;

use crate::trace::{XrayReadTraceHotPath, XrayReadTraceSummary};

/// What one path cost across a session's reads.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct XrayReadTraceEntry {
  /// Times the bytes were read from a source.
  pub reads: u64,
  /// Bytes read in total, which is the redundant figure when `reads` is above one.
  pub bytes: u64,
  /// Bytes the most recent read returned, so unique volume is a sum rather than a division.
  ///
  /// Tracked separately because a write between two reads can change a path's length, and dividing `bytes` by `reads`
  /// would then be quietly wrong rather than obviously so.
  pub size: u64,
}

/// Per-path account of the reads a session actually performed.
///
/// Opt-in and absent unless a caller asks, because it puts a lock on the read path — the one place a sweep spends its
/// time. Present, it answers what no timing can: whether a run reads the same bytes more than once. A full Anomaly
/// verification once read 77.0GB to inspect 18.6GB unique, four animation banks up to 950 times each.
///
/// Counts physical reads only. A read served from [`crate::XrayAssetCache`] performs no I/O and is not one, which is
/// the point: with retention working, a traced sweep's reads fall towards its unique paths.
#[derive(Debug, Default)]
pub struct XrayReadTrace {
  entries: Mutex<HashMap<String, XrayReadTraceEntry>>,
}

impl XrayReadTrace {
  /// Records one physical read of a logical path.
  pub fn record(&self, logical_path: &str, bytes: u64) {
    let mut entries = self.entries.lock().expect("read trace lock is never poisoned");
    let entry: &mut XrayReadTraceEntry = entries.entry(logical_path.to_owned()).or_default();

    entry.reads += 1;
    entry.bytes += bytes;
    entry.size = bytes;
  }

  /// Paths read at least once.
  pub fn len(&self) -> usize {
    self.entries.lock().expect("read trace lock is never poisoned").len()
  }

  /// Whether nothing has been read yet.
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  /// What one path cost, or `None` when it was never read.
  pub fn get(&self, logical_path: &str) -> Option<XrayReadTraceEntry> {
    self
      .entries
      .lock()
      .expect("read trace lock is never poisoned")
      .get(logical_path)
      .copied()
  }

  /// Totals for the session, with the `hottest` most-read paths named.
  ///
  /// The list is capped because a full sweep touches tens of thousands of paths and a report carrying all of them is
  /// unreadable. `paths` is the untruncated count, so a reader can always tell the list is a slice.
  pub fn get_summary(&self, hottest: usize) -> XrayReadTraceSummary {
    let entries = self.entries.lock().expect("read trace lock is never poisoned");

    let mut ranked: Vec<(&String, &XrayReadTraceEntry)> = entries.iter().collect();

    // Reads first, then bytes, then path: two banks read the same number of times order by what they cost, and the
    // path breaks the remaining tie so one run's report can be diffed against another's.
    ranked.sort_by(|(first_path, first), (second_path, second)| {
      second
        .reads
        .cmp(&first.reads)
        .then_with(|| second.bytes.cmp(&first.bytes))
        .then_with(|| first_path.cmp(second_path))
    });

    XrayReadTraceSummary {
      paths: entries.len(),
      reads: entries.values().map(|entry| entry.reads).sum(),
      bytes: entries.values().map(|entry| entry.bytes).sum(),
      unique_bytes: entries.values().map(|entry| entry.size).sum(),
      hottest: ranked
        .into_iter()
        .take(hottest)
        .map(|(path, entry)| XrayReadTraceHotPath {
          path: path.clone(),
          reads: entry.reads,
          bytes: entry.bytes,
        })
        .collect(),
    }
  }
}
