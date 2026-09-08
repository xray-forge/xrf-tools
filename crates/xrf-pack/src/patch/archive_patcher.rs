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

/// Compares two mounted worlds and writes what changed as a volume set the engine mounts over the first.
///
/// A patch works because of one engine rule and one file-layout convention. `CLocatorAPI::Register` overwrites a
/// descriptor whenever a name it already holds is registered again, and `_initialize` recurses over the paths of
/// `fsgame.ltx` in declaration order — so whichever archive is declared last wins. Both the stock `res/fsgame.ltx` and
/// Anomaly's declare `$arch_dir_patches$` immediately before `$game_data$`, which is why `db/patches/` is where a
/// patch goes and why it needs no cooperation from the release it patches.
///
/// Four doors, two of which write. [`Self::compare`] reports the difference and creates nothing; [`Self::patch`] also
/// publishes it. Each has an `_opt` twin taking [`ArchivePatchOptions`]. Whether a run writes is in the name rather
/// than in the options, so a call site says which it is.
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

  /// Compare and publish, reporting to and stoppable through `options`.
  ///
  /// The destination is guarded exactly as a pack guards it: a set already published there is refused unless forced,
  /// and a run that fails or is stopped takes back the volumes it made.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for an unusable configuration, a root that mounted nothing, a scope matching nothing, a
  /// volume declaring an entry point other than the gamedata root, a destination inside either side's roots, and —
  /// under `is_strict` — a base holding entries the target does not; plus any read or write failure.
  pub fn patch_opt(config: &ArchivePatchConfig, options: ArchivePatchOptions) -> XrfResult<ArchivePatchResult> {
    Self::run(config, &options, true)
  }

  /// Compare and report, reporting to and stoppable through `options`.
  ///
  /// # Errors
  ///
  /// The refusals of [`Self::patch_opt`] that a comparison can still meet: everything but the destination guard, which
  /// nothing here would write to.
  pub fn compare_opt(config: &ArchivePatchConfig, options: ArchivePatchOptions) -> XrfResult<ArchivePatchResult> {
    Self::run(config, &options, false)
  }

  /// The comparison both doors share, with `is_publishing` deciding only whether the difference is written.
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

    let result: ArchivePatchResult = ArchivePatchResult {
      outcome: Self::to_outcome(job, &publication),
      added: comparison.added,
      modified: comparison.modified,
      removed: comparison.removed,
      unchanged: comparison.unchanged,
      payloads_read: comparison.payloads_read,
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

  /// Write the carried entries into the destination's volume set.
  ///
  /// Guarding and rollback are the packer's, reached through the same two seams a pack uses, so a patch cannot acquire
  /// its own answer to what a stopped publication leaves behind.
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

  /// What the run has to say for itself, which a cancellation overrides whatever else happened.
  fn to_outcome(job: &JobHandle, publication: &ArchivePatchPublication) -> JobOutcome {
    if job.is_cancelled() {
      return JobOutcome::Cancelled;
    }

    publication
      .get_published()
      .map_or(JobOutcome::Completed, |published| published.outcome)
  }

  /// Refuse a destination inside either side's roots.
  ///
  /// A patch written into a tree it is comparing becomes an input to the next run over the same pair, and
  /// `issues/0130` is that defect for packing. It matters more here: `db/patches/` is both where a patch belongs and
  /// the kind of directory someone would name as a base root, so the mistake is the natural one to make.
  fn require_destination_outside(config: &ArchivePatchConfig, worlds: [&ArchivePatchWorld; 2]) -> XrfResult<()> {
    for world in worlds {
      if let Some(root) = world.find_root_containing(&config.destination) {
        return Err(XrfError::new_invalid_error(format!(
          "Destination '{}' is inside the {} root '{}'. A patch written into a tree it compares becomes part of that \
           tree, so the next run over the same pair would read this run's own output as a difference.",
          format_path(&config.destination),
          world.get_role(),
          format_path(root)
        )));
      }
    }

    Ok(())
  }

  /// Refuse a comparison where a side offered nothing, which is a selection matching nothing rather than an answer.
  ///
  /// An empty *difference* is a fine answer and succeeds — two releases may genuinely match. A side holding nothing is
  /// not: every entry of the other side becomes a difference, so an empty base turns "make me a patch" into "repack
  /// the entire game", and an empty target reports the whole release as deleted. Both are what a mistyped root or a
  /// renamed directory produces, and both look exactly like success.
  ///
  /// Judged per side, because naming which one is empty is the whole value of the refusal. A directory that mounted
  /// but holds nothing is the case `XrayVfs::is_empty` cannot see: it counts mounts, not entries.
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
          "The comparison scope matched no entry on the {} side ({}). A scope naming nothing on one side makes every \
           entry of the other a difference.",
          world.get_role(),
          world.describe_roots()
        )
      } else {
        format!(
          "The {} root(s) hold no entry to compare: {}. Every entry of the other side would become a difference, \
           which is not a comparison.",
          world.get_role(),
          world.describe_roots()
        )
      }));
    }

    Ok(())
  }

  /// Turn entries the format cannot express into a failure, when the caller asked for that.
  ///
  /// Judged after the run rather than before it, so the report naming every one of them exists whichever way this
  /// goes: a caller that catches the error still has the list on the console, and a caller without `is_strict` has it
  /// in the payload.
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
