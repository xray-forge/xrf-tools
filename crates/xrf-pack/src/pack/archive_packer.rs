use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::pack::archive_descriptor_table::ArchiveDescriptorTable;
use crate::pack::archive_pack_config::ArchivePackConfig;
use crate::pack::archive_pack_result::ArchivePackResult;
use crate::pack::archive_pack_source::ArchivePackSource;
use crate::pack::archive_volume_layout::ArchiveVolumeLayout;
use crate::pack::archive_volume_writer::ArchiveVolumeWriter;

/// Writes one volume set from a source tree.
pub struct ArchivePacker;

impl ArchivePacker {
  /// Pack the configured source into `<name>.db<N>` volumes and report what was written.
  ///
  /// Produces the exact layout the engine mounts: an optional header chunk, one data chunk holding every payload, and
  /// a descriptor table whose offsets are absolute positions in the volume.
  ///
  /// `max_volume_size` is a hard maximum on each finished file. An entry is placed only once its
  /// stored-or-compressed payload and its descriptor row are known, and a cap that cannot hold some entry in a
  /// volume of its own is refused rather than exceeded; see [`ArchiveVolumeLayout`].
  pub fn pack(config: &ArchivePackConfig) -> XrfResult<ArchivePackResult> {
    config.validate_for_packing()?;

    let started_at: Instant = Instant::now();
    let source: ArchivePackSource = ArchivePackSource::collect(config)?;

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

    // Both are measured before anything is created, because an unsatisfiable cap is a property of the archive rather
    // than of the file that would first overflow, and refusing it must leave no destination behind.
    let descriptors: ArchiveDescriptorTable = ArchiveDescriptorTable::of_directories(&source.directories)?;
    let layout: ArchiveVolumeLayout = ArchiveVolumeLayout::new(config, &descriptors)?;

    fs::create_dir_all(&config.destination)?;

    let mut writer: ArchiveVolumeWriter = ArchiveVolumeWriter::open(config, layout, descriptors)?;

    for entry in &source.entries {
      writer.write_entry(entry)?;
    }

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
}
