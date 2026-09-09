use std::time::Duration;

use serde::Serialize;
use xrf_job::JobOutcome;

use crate::patch::archive_patch_publication::ArchivePatchPublication;
use crate::patch::compare::{ArchivePatchChange, ArchivePatchOrigin};

/// Archive comparison details and publication outcome. Empty change lists are serialized; unchanged entries are
/// counted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePatchResult {
  /// Whether the run reached the end of its work or was stopped.
  pub outcome: JobOutcome,
  /// Entries only the target holds, which the patch carries.
  pub added: Vec<ArchivePatchChange>,
  /// Entries both hold with differing payloads, which the patch carries from the target.
  pub modified: Vec<ArchivePatchChange>,
  /// Entries only the base holds. Reported but never deleted: the `.db` format cannot encode deletions.
  pub removed: Vec<ArchivePatchChange>,
  /// Entries both sides read identically, counted rather than listed.
  pub unchanged: usize,
  /// Every volume set and loose root the run read from, which each side of each change names by index.
  ///
  /// Shared rather than repeated per entry: a comparison meets a handful of origins and classifies tens of thousands
  /// of entries, so naming one on every side is most of a large report's weight.
  pub origins: Vec<ArchivePatchOrigin>,
  /// Entry pairs requiring a computed checksum. Excludes optional byte-for-byte verification reads.
  pub payloads_read: usize,
  /// Total unpacked size of added and modified target entries, including previews. Matches `size_source` for a
  /// complete publication; archive size is determined when writing.
  pub size_carried: u64,
  /// What was done with the difference.
  pub publication: ArchivePatchPublication,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  /// The share of `duration` spent mounting both sides and deciding what differs.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub compare_duration: Duration,
  /// The share of `duration` spent writing the difference into volumes, zero where none was written.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub pack_duration: Duration,
}

impl ArchivePatchResult {
  /// Entries the patch carries, which is what the volumes hold.
  pub fn get_carried_count(&self) -> usize {
    self.added.len() + self.modified.len()
  }

  /// Whether there are no added or modified entries. Removed entries do not affect this result.
  pub fn is_empty(&self) -> bool {
    self.get_carried_count() == 0
  }
}
