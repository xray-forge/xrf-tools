use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobOutcome, JobScope};
use xrf_utils::format_path;

use crate::pack::config::ArchivePackConfig;
use crate::pack::source::{ArchivePackNameTable, ArchivePackPayloads, ArchivePackSource, ArchivePackSourceCollector};
use crate::pack::volume::{ArchiveDescriptorTable, ArchivePublishedSet, ArchiveVolumeLayout, ArchiveVolumeWriter};
use crate::pack::{ArchivePackNarrator, ArchivePackOptions, ArchivePackResult};
use crate::path::is_inside_directory;

/// Writes one volume set from a source tree.
pub struct ArchivePacker;

impl ArchivePacker {
  /// Pack the configured source into `<name>.db<N>` volumes and report what was written.
  pub fn pack(config: &ArchivePackConfig) -> XrfResult<ArchivePackResult> {
    Self::pack_opt(config, ArchivePackOptions::default())
  }

  /// Volumes of this configuration's set the destination already holds.
  pub fn list_published_volumes(config: &ArchivePackConfig) -> XrfResult<Vec<PathBuf>> {
    Ok(ArchivePublishedSet::read(config)?.get_volumes().to_vec())
  }

  /// Pack the configured source, reporting to and stoppable through `options`.
  pub fn pack_opt(config: &ArchivePackConfig, options: ArchivePackOptions) -> XrfResult<ArchivePackResult> {
    config.validate_for_packing()?;
    Self::require_destination_outside_source(config)?;

    let published: ArchivePublishedSet = Self::guard_destination(config, options.is_forced)?;

    Self::settle_publication(config, &published, Self::pack_into(config, &options))
  }

  /// Refuse a destination that is the source or lies inside it.
  ///
  /// # Errors
  ///
  /// Returns an invalid error naming both paths.
  fn require_destination_outside_source(config: &ArchivePackConfig) -> XrfResult<()> {
    if !is_inside_directory(&config.destination, &config.source) {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "Destination '{}' is inside the source '{}'. Volumes written into the tree being packed become part of it, so the \
       next run would pack this run's own output. Write them beside the source instead, such as a 'db' directory next \
       to 'gamedata'.",
      format_path(&config.destination),
      format_path(&config.source),
    )))
  }

  /// Refuse a destination already holding volumes of this set, unless the run was allowed to replace them.
  ///
  /// # Errors
  ///
  /// Returns an invalid error naming every volume that would be replaced, and any error listing the destination.
  pub(crate) fn guard_destination(config: &ArchivePackConfig, is_forced: bool) -> XrfResult<ArchivePublishedSet> {
    let published: ArchivePublishedSet = ArchivePublishedSet::read(config)?;

    if !published.is_empty() && !is_forced {
      return Err(Self::describe_refusal(config, &published));
    }

    Ok(published)
  }

  /// Leave the destination as this run found it, where the run did not finish and had nothing to replace.
  pub(crate) fn settle_publication(
    config: &ArchivePackConfig,
    published: &ArchivePublishedSet,
    mut outcome: XrfResult<ArchivePackResult>,
  ) -> XrfResult<ArchivePackResult> {
    // Reached only where the destination held no volume of this set to begin with, which is what makes the volumes
    // in it now unambiguously this run's own.
    if published.is_empty() && !Self::is_completed(&outcome) {
      Self::remove_written_volumes(config);

      if let Ok(result) = outcome.as_mut() {
        result.volumes.clear();
        result.volumes_opened.clear();
      }
    }

    outcome
  }

  /// The run itself, from the source walk to the named set.
  fn pack_into(config: &ArchivePackConfig, options: &ArchivePackOptions) -> XrfResult<ArchivePackResult> {
    let started_at: Instant = Instant::now();
    let job: &JobHandle = &options.job;
    let narrator: ArchivePackNarrator = ArchivePackNarrator::new(&options.output);

    narrator.describe_settings(config);

    let source: ArchivePackSource = {
      let collecting: JobScope = job.enter(ArchivePackOptions::PHASE_COLLECT, None);

      ArchivePackSourceCollector::collect_opt(config, &collecting, narrator.is_recording())?
    };

    let collected_at: Duration = started_at.elapsed();

    narrator.describe_selection(source.names.get_directories(), &source.omitted);

    // xrCompress refuses an empty file list too. Saying so here beats leaving the caller to puzzle out
    // a complaint from the codec about an empty descriptor table.
    if source.names.get_files().is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Nothing to pack from '{}': {} file(s) matched, {} skipped by the configured rules",
        format_path(&config.source),
        source.names.get_files().len(),
        source.omitted.get_count()
      )));
    }

    // Nothing has been created yet, so stopping here leaves the destination untouched. Worth the check: walking a
    // large source tree is minutes of work on its own, and this is the last boundary where cancelling costs nothing.
    if job.is_cancelled() {
      return Ok(Self::describe_cancelled(
        ArchivePackResult::default(),
        &source,
        started_at,
        collected_at,
      ));
    }

    // A walked source reads its payloads off the host, at the path each entry was reached by.
    let mut result: ArchivePackResult = Self::write_set(
      config,
      &source.names,
      &ArchivePackPayloads::Host,
      job,
      &narrator,
      ArchivePackOptions::PHASE_WRITE,
    )?;

    if result.outcome == JobOutcome::Cancelled {
      return Ok(Self::describe_cancelled(result, &source, started_at, collected_at));
    }

    let written_at: Duration = started_at.elapsed();

    result.files_skipped = source.omitted.get_count();

    result.measure(started_at, collected_at, written_at);

    Ok(result)
  }

  /// Write one already-decided name table into the destination's volume set.
  ///
  /// # Errors
  ///
  /// Returns an error for a volume cap the archive cannot be written within, and for any read or write failure.
  pub(crate) fn write_set(
    config: &ArchivePackConfig,
    names: &ArchivePackNameTable,
    payloads: &ArchivePackPayloads,
    job: &JobHandle,
    narrator: &ArchivePackNarrator,
    write_phase: &'static str,
  ) -> XrfResult<ArchivePackResult> {
    // Both are measured before anything is created, because an unsatisfiable cap is a property of the archive rather
    // than of the file that would first overflow, and refusing it must leave no destination behind.
    let descriptors: ArchiveDescriptorTable = ArchiveDescriptorTable::of_directories(names.get_directories())?;
    let layout: ArchiveVolumeLayout = ArchiveVolumeLayout::new(config, &descriptors)?;

    fs::create_dir_all(&config.destination)?;

    let mut writer: ArchiveVolumeWriter = ArchiveVolumeWriter::open(config, narrator, payloads, layout, descriptors)?;

    {
      let writing: JobScope = job.enter(write_phase, Some(names.get_files().len() as u64));

      for entry in names.get_files() {
        // Between entries: a payload half-written into a volume would leave the descriptor table describing bytes
        // that are not there, which is worse than a volume that simply ends early.
        if job.is_cancelled() {
          let mut abandoned: ArchivePackResult = writer.abandon();

          abandoned.outcome = JobOutcome::Cancelled;

          return Ok(abandoned);
        }

        // Sequential, so naming the current entry is meaningful here in a way it is not for a parallel unpack.
        job.set_detail(Some(entry.name.clone()));

        writer.write_entry(entry)?;
        writing.advance();
      }
    }

    // Bound rather than scoped in a block, because this phase covers everything left: closing the last volume, naming
    // the set, and measuring. It ends when the function does.
    //
    // The binding is what holds it open. `let _ = job.enter(..)` would drop the scope immediately and leave the run
    // reporting the phase it had already left, so keep the name even though nothing reads it.
    let _finalizing: JobScope = job.enter(ArchivePackOptions::PHASE_FINALIZE, None);

    job.set_detail(None);

    // The writer reports what it saw writing; what the clock knows is added by the caller. The entry count is set
    // here rather than there because the name table is what defines it, and a caller that forgot would report zero
    // beside real per-outcome counts.
    let mut result: ArchivePackResult = writer.finish()?;

    result.files_total = names.get_files().len();

    // Only now is the volume count known, so a set that stayed single drops its index.
    if let [only] = result.volumes.as_slice() {
      let renamed: PathBuf = config.destination.join(config.single_volume_name());

      fs::rename(only, &renamed)?;

      result.volumes = vec![renamed.clone()];
      result.volumes_opened = vec![renamed];
    }

    Ok(result)
  }

  /// Whether a run reached the end of its work.
  fn is_completed(outcome: &XrfResult<ArchivePackResult>) -> bool {
    matches!(outcome, Ok(result) if result.outcome == JobOutcome::Completed)
  }

  /// Take back every volume of the set now in the destination.
  fn remove_written_volumes(config: &ArchivePackConfig) {
    match ArchivePublishedSet::read(config) {
      Ok(written) => written.remove_all(),
      Err(error) => log::warn!(
        "Could not list '{}' to remove what a failed archive pack wrote: {error}",
        format_path(&config.destination)
      ),
    }
  }

  /// Why a run that would replace an existing set stopped before doing anything.
  fn describe_refusal(config: &ArchivePackConfig, published: &ArchivePublishedSet) -> XrfError {
    let names: Vec<String> = published
      .get_volumes()
      .iter()
      .map(|volume| {
        volume
          .file_name()
          .unwrap_or(volume.as_os_str())
          .to_string_lossy()
          .into_owned()
      })
      .collect();

    XrfError::new_invalid_error(format!(
      "Destination '{}' already holds {} volume(s) of the archive set '{}': {}. Packing replaces them and cannot put \
       them back if it fails partway, so publishing over them has to be asked for.",
      format_path(&config.destination),
      names.len(),
      config.single_volume_name(),
      names.join(", ")
    ))
  }

  /// What a run that stopped has to say for itself.
  fn describe_cancelled(
    mut result: ArchivePackResult,
    source: &ArchivePackSource,
    started_at: Instant,
    collected_at: Duration,
  ) -> ArchivePackResult {
    result.outcome = JobOutcome::Cancelled;
    result.files_total = source.names.get_files().len();
    result.files_skipped = source.omitted.get_count();

    result.measure_stopped(started_at, collected_at);

    result
  }
}
