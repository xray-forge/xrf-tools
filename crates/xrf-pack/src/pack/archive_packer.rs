use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobOutcome, JobScope};
use xrf_utils::format_path;

use crate::pack::config::ArchivePackConfig;
use crate::pack::source::{ArchivePackSource, ArchivePackSourceCollector};
use crate::pack::volume::{ArchiveDescriptorTable, ArchivePublishedSet, ArchiveVolumeLayout, ArchiveVolumeWriter};
use crate::pack::{
  ArchivePackNarrator, ArchivePackOptions, ArchivePackResult, PACK_PHASE_COLLECT, PACK_PHASE_FINALIZE, PACK_PHASE_WRITE,
};

/// Writes one volume set from a source tree.
pub struct ArchivePacker;

impl ArchivePacker {
  /// Pack the configured source into `<name>.db<N>` volumes and report what was written.
  ///
  /// The plain door. A caller that wants to watch the run, be able to stop it, or replace a set the destination
  /// already holds uses [`Self::pack_opt`].
  pub fn pack(config: &ArchivePackConfig) -> XrfResult<ArchivePackResult> {
    Self::pack_opt(config, ArchivePackOptions::default())
  }

  /// Volumes of this configuration's set the destination already holds.
  ///
  /// What [`Self::pack_opt`] refuses over, published so a caller can ask before starting rather than be told once the
  /// source tree has been walked. Asking twice is not a race worth closing: this is a courtesy for someone about to
  /// commit to minutes of work, and the refusal inside the run is the guarantee.
  pub fn list_published_volumes(config: &ArchivePackConfig) -> XrfResult<Vec<PathBuf>> {
    Ok(ArchivePublishedSet::read(config)?.get_volumes().to_vec())
  }

  /// Pack the configured source, reporting to and stoppable through `options`.
  ///
  /// Produces the exact layout the engine mounts: an optional header chunk, one data chunk holding every payload, and
  /// a descriptor table whose offsets are absolute positions in the volume.
  ///
  /// `max_volume_size` is a hard maximum on each finished file. An entry is placed only once its
  /// stored-or-compressed payload and its descriptor row are known, and a cap that cannot hold some entry in a
  /// volume of its own is refused rather than exceeded; see [`ArchiveVolumeLayout`].
  ///
  /// A destination already holding volumes of this set is refused unless `options` forces it, because publication is
  /// destructive: volumes are opened at their final names, so a run that fails or is stopped partway cannot give back
  /// what it overwrote. Refused before the source tree is walked, so the run nobody wanted is also the cheapest one.
  ///
  /// Where nothing was refused, nothing is left behind either. Every volume of the set in the destination afterwards
  /// was written by this run, so a failure or a cancellation removes them and leaves the destination as it was found.
  /// A forced run is the exception, and the reason [`ArchivePackResult::volumes_opened`] exists: there, what is on
  /// disk cannot be told apart from what the run replaced, so it is reported rather than deleted.
  ///
  /// What the run decides is said as it is decided, through the options' output at verbose level, so a transcript of
  /// a run that stopped already names the volume being written and the last entry it reached. Saying it changes no
  /// byte of what is written.
  pub fn pack_opt(config: &ArchivePackConfig, options: ArchivePackOptions) -> XrfResult<ArchivePackResult> {
    config.validate_for_packing()?;

    let published: ArchivePublishedSet = ArchivePublishedSet::read(config)?;

    if !published.is_empty() && !options.is_forced {
      return Err(Self::describe_refusal(config, &published));
    }

    let mut outcome: XrfResult<ArchivePackResult> = Self::pack_into(config, &options);

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
  ///
  /// Split from [`Self::pack_opt`] so one place decides what a run that did not finish leaves behind, rather than
  /// every early return deciding it again and one of them forgetting.
  fn pack_into(config: &ArchivePackConfig, options: &ArchivePackOptions) -> XrfResult<ArchivePackResult> {
    let started_at: Instant = Instant::now();
    let job: &JobHandle = &options.job;
    let narrator: ArchivePackNarrator = ArchivePackNarrator::new(&options.output);

    narrator.describe_settings(config);

    let source: ArchivePackSource = {
      let collecting: JobScope = job.enter(PACK_PHASE_COLLECT, None);

      ArchivePackSourceCollector::collect_opt(config, &collecting, narrator.is_recording())?
    };

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
      ));
    }

    // Both are measured before anything is created, because an unsatisfiable cap is a property of the archive rather
    // than of the file that would first overflow, and refusing it must leave no destination behind.
    let descriptors: ArchiveDescriptorTable = ArchiveDescriptorTable::of_directories(source.names.get_directories())?;
    let layout: ArchiveVolumeLayout = ArchiveVolumeLayout::new(config, &descriptors)?;

    fs::create_dir_all(&config.destination)?;

    let mut writer: ArchiveVolumeWriter = ArchiveVolumeWriter::open(config, &narrator, layout, descriptors)?;

    {
      let writing: JobScope = job.enter(PACK_PHASE_WRITE, Some(source.names.get_files().len() as u64));

      for entry in source.names.get_files() {
        // Between entries: a payload half-written into a volume would leave the descriptor table describing bytes
        // that are not there, which is worse than a volume that simply ends early.
        if job.is_cancelled() {
          return Ok(Self::describe_cancelled(writer.abandon(), &source, started_at));
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
    let _finalizing: JobScope = job.enter(PACK_PHASE_FINALIZE, None);

    job.set_detail(None);

    // The writer reports what it saw writing; what the source and the clock know is added here.
    let mut result: ArchivePackResult = writer.finish()?;

    result.files_total = source.names.get_files().len();
    result.files_skipped = source.omitted.get_count();

    // Only now is the volume count known, so a set that stayed single drops its index.
    if let [only] = result.volumes.as_slice() {
      let renamed: PathBuf = config.destination.join(config.single_volume_name());

      fs::rename(only, &renamed)?;

      result.volumes = vec![renamed.clone()];
      result.volumes_opened = vec![renamed];
    }

    result.measure(started_at);

    Ok(result)
  }

  /// Whether a run reached the end of its work.
  fn is_completed(outcome: &XrfResult<ArchivePackResult>) -> bool {
    matches!(outcome, Ok(result) if result.outcome == JobOutcome::Completed)
  }

  /// Take back every volume of the set now in the destination.
  ///
  /// Listed again rather than read off the result: a run can fail before it has a result to report, and the volume it
  /// failed inside is on disk either way. Listing the destination is the one answer that covers both.
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
  ///
  /// The counts describe what was reached, never what was asked for: `files_total` is the size of the job, and the
  /// volume lists are what is actually on disk. Naming the set is intentionally skipped — a single volume keeps its
  /// index, because renaming it would dress an incomplete set as a finished one.
  fn describe_cancelled(
    mut result: ArchivePackResult,
    source: &ArchivePackSource,
    started_at: Instant,
  ) -> ArchivePackResult {
    result.outcome = JobOutcome::Cancelled;
    result.files_total = source.names.get_files().len();
    result.files_skipped = source.omitted.get_count();
    result.measure(started_at);

    result
  }
}
