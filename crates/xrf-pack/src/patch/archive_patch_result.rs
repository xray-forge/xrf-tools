use std::time::Duration;

use serde::Serialize;
use xrf_job::JobOutcome;

use crate::patch::archive_patch_publication::ArchivePatchPublication;
use crate::patch::compare::ArchivePatchChange;

/// What one patch run compared, and what became of the difference.
///
/// The three change lists are always present, so `[]` reads as "none of this kind" rather than as "not reported" —
/// the rule a coverage report's `skippedMounts` already set. `unchanged` is a count and not a list: it is the whole of
/// a mature project and answers nothing a reader asked.
///
/// This deliberately carries the per-entry detail that [`crate::ArchivePackResult`] refuses. A packer's answer is that
/// it packed, so a file tree there was noise; a comparison's answer *is* which files differ, and a report withholding
/// it would report nothing.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePatchResult {
  /// Whether the run reached the end of its work or was stopped.
  pub outcome: JobOutcome,
  /// Entries only the target holds, which the patch carries.
  pub added: Vec<ArchivePatchChange>,
  /// Entries both hold with differing payloads, which the patch carries from the target.
  pub modified: Vec<ArchivePatchChange>,
  /// Entries only the base holds.
  ///
  /// Reported and never encoded. The `.db` format has no tombstone and `CLocatorAPI::Register` only ever overwrites,
  /// so no patch can make these stop existing.
  pub removed: Vec<ArchivePatchChange>,
  /// Entries both sides read identically, counted rather than listed.
  pub unchanged: usize,
  /// Entries whose payload had to be read to classify them.
  ///
  /// Zero for an archive-to-archive comparison, which decides everything from two name tables. Worth reporting
  /// because it is the whole cost difference between the cheap and the expensive shape of the same run.
  pub payloads_read: usize,
  /// What was done with the difference.
  pub publication: ArchivePatchPublication,
  #[serde(with = "xrf_utils::duration_ms")]
  pub duration: Duration,
  /// The share of `duration` spent mounting both sides and deciding what differs.
  #[serde(with = "xrf_utils::duration_ms")]
  pub compare_duration: Duration,
  /// The share of `duration` spent writing the difference into volumes, zero where none was written.
  #[serde(with = "xrf_utils::duration_ms")]
  pub pack_duration: Duration,
}

impl ArchivePatchResult {
  /// Entries the patch carries, which is what the volumes hold.
  pub fn get_carried_count(&self) -> usize {
    self.added.len() + self.modified.len()
  }

  /// Whether the comparison found nothing to carry.
  ///
  /// A true and complete answer, not a failure: two releases may genuinely match. It is an empty *selection* — a root
  /// that mounted nothing, a scope matching nothing — that is refused instead, because that is the shape which makes a
  /// release gate vacuous without anyone noticing.
  pub fn is_empty(&self) -> bool {
    self.get_carried_count() == 0
  }
}
