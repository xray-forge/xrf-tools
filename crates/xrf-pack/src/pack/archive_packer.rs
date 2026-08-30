use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobOutcome, JobScope};
use xrf_utils::format_path;

use crate::pack::archive_descriptor_table::ArchiveDescriptorTable;
use crate::pack::archive_pack_config::ArchivePackConfig;
use crate::pack::archive_pack_options::{
  ArchivePackOptions, PACK_PHASE_COLLECT, PACK_PHASE_FINALIZE, PACK_PHASE_WRITE,
};
use crate::pack::archive_pack_result::ArchivePackResult;
use crate::pack::archive_pack_source::ArchivePackSource;
use crate::pack::archive_volume_layout::ArchiveVolumeLayout;
use crate::pack::archive_volume_writer::ArchiveVolumeWriter;

/// Writes one volume set from a source tree.
pub struct ArchivePacker;

impl ArchivePacker {
  /// Pack the configured source into `<name>.db<N>` volumes and report what was written.
  ///
  /// The plain door. A caller that wants to watch the run or be able to stop it uses [`Self::pack_opt`].
  pub fn pack(config: &ArchivePackConfig) -> XrfResult<ArchivePackResult> {
    Self::pack_opt(config, ArchivePackOptions::default())
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
  /// Cancellation lands between entries and removes nothing. What has been written stays: the volume paths were
  /// created with `File::create`, so they replaced whatever stood there the moment the run began, and no rule here can
  /// separate what this run wrote from what it overwrote. The result names every path it opened for exactly that
  /// reason. Until packing stages its output elsewhere and promotes it, a cancelled pack leaves an unusable set.
  pub fn pack_opt(config: &ArchivePackConfig, options: ArchivePackOptions) -> XrfResult<ArchivePackResult> {
    config.validate_for_packing()?;

    let started_at: Instant = Instant::now();
    let job: JobHandle = options.job;

    let source: ArchivePackSource = {
      let collecting: JobScope = job.enter(PACK_PHASE_COLLECT, None);

      ArchivePackSource::collect_opt(config, &collecting)?
    };

    // xrCompress refuses an empty file list too. Saying so here beats leaving the caller to puzzle out
    // a complaint from the codec about an empty descriptor table.
    if source.entries.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Nothing to pack from '{}': {} file(s) matched, {} skipped by the configured rules",
        format_path(&config.source),
        source.entries.len(),
        source.skipped
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
    let descriptors: ArchiveDescriptorTable = ArchiveDescriptorTable::of_directories(&source.directories)?;
    let layout: ArchiveVolumeLayout = ArchiveVolumeLayout::new(config, &descriptors)?;

    fs::create_dir_all(&config.destination)?;

    let mut writer: ArchiveVolumeWriter = ArchiveVolumeWriter::open(config, layout, descriptors)?;

    {
      let writing: JobScope = job.enter(PACK_PHASE_WRITE, Some(source.entries.len() as u64));

      for entry in &source.entries {
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

    result.files_total = source.entries.len();
    result.files_skipped = source.skipped;

    // Only now is the volume count known, so a set that stayed single drops its index.
    if let [only] = result.volumes.as_slice() {
      let renamed: PathBuf = config.destination.join(config.single_volume_name());

      fs::rename(only, &renamed)?;

      result.volumes = vec![renamed];
    }

    result.duration = started_at.elapsed();

    Ok(result)
  }

  /// What a run that stopped has to say for itself.
  ///
  /// The counts describe what was reached, never what was asked for: `files_total` is the size of the job, and the
  /// volume lists are what is actually on disk. Naming the set is deliberately skipped — a single volume keeps its
  /// index, because renaming it would dress an incomplete set as a finished one.
  fn describe_cancelled(
    mut result: ArchivePackResult,
    source: &ArchivePackSource,
    started_at: Instant,
  ) -> ArchivePackResult {
    result.outcome = JobOutcome::Cancelled;
    result.files_total = source.entries.len();
    result.files_skipped = source.skipped;
    result.duration = started_at.elapsed();

    result
  }
}
