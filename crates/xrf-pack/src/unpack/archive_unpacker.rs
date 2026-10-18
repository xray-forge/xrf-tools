use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use xrf_archive::{ArchiveFileDescriptor, ArchiveOpenVolumes, ArchiveProject};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::path::{relative_to_prefix, to_host_relative};
use crate::unpack::archive_extract_options::ArchiveExtractOptions;
use crate::unpack::archive_extract_result::{ArchiveExtractDirectoryResult, ArchiveExtractResult};
use crate::unpack::archive_unpack_result::ArchiveUnpackResult;
use crate::unpack::rooted_destination::RootedDestination;
use crate::unpack::tree_entry::TreeEntry;
use crate::unpack::tree_write_summary::TreeWriteSummary;
use crate::unpack::tree_writer::TreeWriter;

/// Writes the contents of an archive project back out to a directory.
pub struct ArchiveUnpacker;

impl ArchiveUnpacker {
  /// Write every file in the project beneath a destination root.
  pub fn unpack<P: AsRef<Path>>(project: &ArchiveProject, destination: P) -> XrfResult<ArchiveUnpackResult> {
    Self::unpack_opt(project, destination, ArchiveExtractOptions::default())
  }

  /// Write every file in the project beneath a destination root, as `options` asks.
  pub fn unpack_opt<P: AsRef<Path>>(
    project: &ArchiveProject,
    destination: P,
    options: ArchiveExtractOptions,
  ) -> XrfResult<ArchiveUnpackResult> {
    let destination: RootedDestination = RootedDestination::new(destination.as_ref());
    let job = &options.job;

    destination.create_root()?;

    // Opened before the write phase and shared by reference: every worker reads by position, so one handle per volume
    // serves the whole run without a cursor to race for.
    let volumes: ArchiveOpenVolumes<'_> = project.open_volumes()?;

    let entries: Vec<TreeEntry<&ArchiveFileDescriptor>> = project
      .files
      .values()
      .map(|descriptor| {
        Ok(TreeEntry {
          source: descriptor,
          relative: Self::build_relative_path(volumes.get_unpack_root_of(descriptor)?, descriptor)?,
          is_directory: descriptor.is_directory,
        })
      })
      .collect::<XrfResult<Vec<TreeEntry<&ArchiveFileDescriptor>>>>()?;

    let summary: TreeWriteSummary = Self::write_entries(&destination, &volumes, &entries, &options)?;

    Ok(Self::describe_result(
      project,
      destination.get_root(),
      &summary,
      job.elapsed(),
    ))
  }

  /// Write every archived file under one directory to a destination root.
  pub fn extract_directory<P: AsRef<Path>>(
    project: &ArchiveProject,
    prefix: &str,
    destination: P,
  ) -> XrfResult<ArchiveExtractDirectoryResult> {
    Self::extract_directory_opt(project, prefix, destination, ArchiveExtractOptions::default())
  }

  /// Write every archived file under one directory to a destination root, as `options` asks.
  pub fn extract_directory_opt<P: AsRef<Path>>(
    project: &ArchiveProject,
    prefix: &str,
    destination: P,
    options: ArchiveExtractOptions,
  ) -> XrfResult<ArchiveExtractDirectoryResult> {
    let normalized: String = prefix.trim_end_matches(['\\', '/']).to_string();
    let destination: RootedDestination = RootedDestination::new(destination.as_ref());

    destination.create_root()?;

    // One handle per volume here too: a directory extraction writes many entries of the same set.
    let volumes: ArchiveOpenVolumes<'_> = project.open_volumes()?;

    // Selected before anything is written, so the run knows how much it is about to do. The alternative - filtering
    // inside the write loop - leaves the total unknowable until the end, which is exactly when it stops being useful.
    let selected: Vec<TreeEntry<&ArchiveFileDescriptor>> = project
      .files
      .values()
      .filter_map(|descriptor| relative_to_prefix(&descriptor.name, &normalized).map(|relative| (descriptor, relative)))
      .map(|(descriptor, relative)| {
        Ok(TreeEntry {
          source: descriptor,
          relative: to_host_relative(relative)?,
          is_directory: descriptor.is_directory,
        })
      })
      .collect::<XrfResult<Vec<TreeEntry<&ArchiveFileDescriptor>>>>()?;

    if selected.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "Cannot extract '{normalized}' - no files in the archive are under it."
      )));
    }

    let summary: TreeWriteSummary = Self::write_entries(&destination, &volumes, &selected, &options)?;

    Ok(ArchiveExtractDirectoryResult {
      prefix: normalized,
      destination: format_path(destination.get_root()).to_string(),
      outcome: summary.outcome,
      extracted_count: summary.files,
      size: summary.bytes,
    })
  }

  /// Write one archived file to an exact path of the caller's choosing.
  pub fn extract_file<P: AsRef<Path>>(
    project: &ArchiveProject,
    name: &str,
    destination: P,
  ) -> XrfResult<ArchiveExtractResult> {
    let descriptor: &ArchiveFileDescriptor = project.files.get(name).ok_or_else(|| {
      XrfError::new_not_found_error(format!("Cannot extract '{name}' - no such file in the archive."))
    })?;

    if descriptor.is_directory {
      return Err(XrfError::new_invalid_error(format!(
        "Cannot extract '{}' as a file - it is a directory record.",
        descriptor.name
      )));
    }

    if let Some(parent) = destination.as_ref().parent() {
      fs::create_dir_all(parent)?;
    }

    // One entry, so the volumes are opened for the length of this call: the same seam a whole unpack holds open,
    // asked for one file.
    project
      .open_volumes()?
      .write_contents(&mut Self::create_target(destination.as_ref())?, descriptor)?;

    Ok(ArchiveExtractResult {
      name: descriptor.name.to_string(),
      destination: format_path(destination.as_ref()).to_string(),
      size: descriptor.size_real as u64,
    })
  }

  /// Writes the selected entries through the shared tree writer, reading each payload out of its volume.
  fn write_entries(
    destination: &RootedDestination,
    volumes: &ArchiveOpenVolumes<'_>,
    entries: &[TreeEntry<&ArchiveFileDescriptor>],
    options: &ArchiveExtractOptions,
  ) -> XrfResult<TreeWriteSummary> {
    TreeWriter::write(destination, entries, &options.job, |descriptor, target| {
      volumes.write_contents(target, descriptor)?;

      Ok(u64::from(descriptor.size_real))
    })
  }

  fn describe_result(
    project: &ArchiveProject,
    destination: &Path,
    summary: &TreeWriteSummary,
    unpacked_at: Duration,
  ) -> ArchiveUnpackResult {
    ArchiveUnpackResult {
      archives: project
        .archives
        .iter()
        .map(|it| format_path(&it.path).to_string())
        .collect(),
      destination: format_path(destination).to_string(),
      duration: unpacked_at,
      outcome: summary.outcome,
      files_total: project.files.len(),
      files_unpacked: summary.files,
      prepare_duration: summary.prepared_at,
      unpack_duration: unpacked_at.saturating_sub(summary.prepared_at),
      unpacked_size: summary.bytes,
    }
  }

  fn create_target(path: &Path) -> XrfResult<File> {
    Ok(
      File::options()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?,
    )
  }

  /// Where one archived entry lands, relative to a destination root.
  fn build_relative_path(unpack_root: &Path, descriptor: &ArchiveFileDescriptor) -> XrfResult<PathBuf> {
    let mut path: PathBuf = to_host_relative(&unpack_root.to_string_lossy())?;

    path.push(to_host_relative(&descriptor.name)?);

    Ok(path)
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use xrf_archive::ArchiveFileDescriptor;

  use super::ArchiveUnpacker;

  #[test]
  fn an_entry_lands_under_its_volumes_root_as_a_real_tree() {
    let descriptor: ArchiveFileDescriptor = ArchiveFileDescriptor::new(0, "configs\\system.ltx".into(), 0, 0, 0);

    assert_eq!(
      ArchiveUnpacker::build_relative_path(Path::new("gamedata\\"), &descriptor).expect("safe archive path"),
      PathBuf::from("gamedata").join("configs").join("system.ltx")
    );
  }
}
