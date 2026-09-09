use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobOutcome};
use xrf_utils::format_path;

use crate::pack::config::ArchivePackConfig;
use crate::pack::source::{ArchivePackEntry, ArchivePackNameTable, ArchivePackPayloads};
use crate::pack::volume::ArchivePublishedSet;
use crate::pack::{ArchivePackNarrator, ArchivePackResult, ArchivePacker};
use crate::patch::archive_patch_publication::ArchivePatchPublication;
use crate::patch::compare::{ArchivePatchChange, ArchivePatchComparison};
use crate::patch::config::{ArchivePatchConfig, ArchivePatchScope};
use crate::patch::world::{ArchivePatchRole, ArchivePatchWorld};
use crate::patch::{
  ArchivePatchNarrator, ArchivePatchOptions, ArchivePatchResult, PATCH_PHASE_PACK, to_patch_directory_rows,
};

/// Compares two roots and publishes added or modified target entries as archive volumes.
///
/// Mount patch volumes after the base archives so their entries take precedence. Use [`Self::compare`] for a
/// preview and [`Self::patch`] to publish.
pub struct ArchivePatcher;

impl ArchivePatcher {
  /// Compare two worlds and publish the difference.
  pub fn patch(config: &ArchivePatchConfig) -> XrfResult<ArchivePatchResult> {
    Self::patch_opt(config, ArchivePatchOptions::default())
  }

  /// Compare two worlds and report the difference, writing nothing.
  pub fn compare(config: &ArchivePatchConfig) -> XrfResult<ArchivePatchResult> {
    Self::compare_opt(config, ArchivePatchOptions::default())
  }

  /// Compares and publishes with progress reporting and cancellation.
  ///
  /// Replacing an existing set requires force. Unforced runs roll back failed or cancelled writes; forced runs
  /// cannot restore overwritten volumes.
  ///
  /// # Errors
  ///
  /// Rejects invalid configuration, empty selections, non-gamedata archive entry points, and destinations
  /// inside either root. Propagates read and write errors. Strict mode rejects removals after publication.
  pub fn patch_opt(config: &ArchivePatchConfig, options: ArchivePatchOptions) -> XrfResult<ArchivePatchResult> {
    Self::run(config, &options, true)
  }

  /// Compares without writing, with progress reporting and cancellation.
  ///
  /// # Errors
  ///
  /// Returns comparison and configuration errors described by [`Self::patch_opt`]. Skips destination
  /// containment and overwrite checks.
  pub fn compare_opt(config: &ArchivePatchConfig, options: ArchivePatchOptions) -> XrfResult<ArchivePatchResult> {
    Self::run(config, &options, false)
  }

  fn run(
    config: &ArchivePatchConfig,
    options: &ArchivePatchOptions,
    is_publishing: bool,
  ) -> XrfResult<ArchivePatchResult> {
    config.validate_for_patching()?;

    let started_at: Instant = Instant::now();
    let job: &JobHandle = &options.job;
    let narrator: ArchivePatchNarrator = ArchivePatchNarrator::new(&options.output);
    let scope: ArchivePatchScope = config.to_scope()?;

    let base: ArchivePatchWorld = ArchivePatchWorld::mount(&config.base, ArchivePatchRole::Base)?;
    let target: ArchivePatchWorld = ArchivePatchWorld::mount(&config.target, ArchivePatchRole::Target)?;

    if is_publishing {
      Self::require_destination_outside(config, [&base, &target])?;
    }

    narrator.describe_settings(config, &base, &target);

    let comparison: ArchivePatchComparison =
      ArchivePatchComparison::of(&base, &target, &scope, job, options.is_verifying_payload)?;

    Self::require_both_sides_hold_entries(&comparison, [&base, &target], &scope)?;
    narrator.describe_comparison(&comparison);

    let compare_duration: Duration = started_at.elapsed();
    let published_at: Instant = Instant::now();
    let publication: ArchivePatchPublication = if !is_publishing || job.is_cancelled() {
      ArchivePatchPublication::Compared
    } else if comparison.get_carried_count() == 0 {
      ArchivePatchPublication::Unnecessary
    } else {
      ArchivePatchPublication::Published(Self::publish(config, &comparison, &target, options, &narrator)?)
    };

    // Read before the lists move into the result, since it is derived from them.
    let size_carried: u64 = comparison.get_carried_size();

    let result: ArchivePatchResult = ArchivePatchResult {
      outcome: Self::to_outcome(job, &publication),
      added: comparison.added,
      modified: comparison.modified,
      removed: comparison.removed,
      unchanged: comparison.unchanged,
      origins: comparison.origins.into_entries(),
      payloads_read: comparison.payloads_read,
      size_carried,
      pack_duration: if publication.is_published() {
        published_at.elapsed()
      } else {
        Duration::ZERO
      },
      publication,
      compare_duration,
      duration: started_at.elapsed(),
    };

    Self::require_no_removals(&result.removed, options.is_strict)?;

    Ok(result)
  }

  /// Publishes selected entries using the packer's destination guard and rollback policy.
  fn publish(
    config: &ArchivePatchConfig,
    comparison: &ArchivePatchComparison,
    target: &ArchivePatchWorld,
    options: &ArchivePatchOptions,
    narrator: &ArchivePatchNarrator,
  ) -> XrfResult<ArchivePackResult> {
    let publication: ArchivePackConfig = config.to_publication();
    let carried: Vec<&str> = comparison.to_carried_names();
    let names: ArchivePackNameTable = ArchivePackNameTable::register(
      &publication,
      carried.iter().copied().map(ArchivePackEntry::of_mounted).collect(),
      to_patch_directory_rows(&carried),
    )?;

    let guarded: ArchivePublishedSet = ArchivePacker::guard_destination(&publication, options.is_forced)?;
    let pack_narrator: ArchivePackNarrator = ArchivePackNarrator::new(narrator.get_output());
    let written: XrfResult<ArchivePackResult> = ArchivePacker::write_set(
      &publication,
      &names,
      &ArchivePackPayloads::Mounted(target.get_vfs()),
      &options.job,
      &pack_narrator,
      PATCH_PHASE_PACK,
    );

    ArchivePacker::settle_publication(&publication, &guarded, written)
  }

  /// Returns the publication outcome, with cancellation taking precedence.
  fn to_outcome(job: &JobHandle, publication: &ArchivePatchPublication) -> JobOutcome {
    if job.is_cancelled() {
      return JobOutcome::Cancelled;
    }

    publication
      .get_published()
      .map_or(JobOutcome::Completed, |published| published.outcome)
  }

  /// Refuse a destination inside either side's roots.
  fn require_destination_outside(config: &ArchivePatchConfig, worlds: [&ArchivePatchWorld; 2]) -> XrfResult<()> {
    for world in worlds {
      if world.contains(&config.destination) {
        return Err(XrfError::new_invalid_error(format!(
          "Destination '{}' is inside the {} root '{}'. A patch written into a tree it compares becomes part of that \
           tree, so the next run over the same pair would read this run's own output as a difference.",
          format_path(&config.destination),
          world.get_role(),
          format_path(world.get_root())
        )));
      }
    }

    Ok(())
  }

  /// Rejects a side with no entries in scope, which would classify the entire other side as changed.
  fn require_both_sides_hold_entries(
    comparison: &ArchivePatchComparison,
    worlds: [&ArchivePatchWorld; 2],
    scope: &ArchivePatchScope,
  ) -> XrfResult<()> {
    for world in worlds {
      if comparison.get_listed_count(world.get_role()) > 0 {
        continue;
      }

      return Err(XrfError::new_invalid_error(if scope.is_narrowed() {
        format!(
          "The comparison scope matched no entry on the {} side ('{}'). A scope naming nothing on one side makes \
           every entry of the other a difference.",
          world.get_role(),
          world.describe_root()
        )
      } else {
        format!(
          "The {} root '{}' holds no entry to compare. Every entry of the other side would become a difference, \
           which is not a comparison.",
          world.get_role(),
          world.describe_root()
        )
      }));
    }

    Ok(())
  }

  /// Rejects removals in strict mode after comparison reporting and any publication.
  fn require_no_removals(removed: &[ArchivePatchChange], is_strict: bool) -> XrfResult<()> {
    if !is_strict || removed.is_empty() {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "{} entry(s) the base holds are absent from the target, and a '.db' patch cannot express a deletion: \
       'CLocatorAPI::Register' overwrites a descriptor and never removes one. They are listed in the report; ship a \
       tree rather than a patch to remove them, or drop the strict check to publish anyway.",
      removed.len()
    )))
  }
}
