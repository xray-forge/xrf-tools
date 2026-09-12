use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobOutcome, JobScope};
use xrf_utils::format_path;
use xrf_vfs::{XrayAsset, XrayMountedEntry, XrayProbe};

use crate::path::{relative_to_prefix, to_host_relative};
use crate::unpack::archive_extract_options::{ArchiveExtractOptions, EXTRACT_PHASE_WRITE};
use crate::unpack::archive_extract_result::{ArchiveExtractDirectoryResult, ArchiveExtractResult};
use crate::unpack::rooted_destination::RootedDestination;

/// Writes what a mounted world resolves out to a directory.
///
/// The counterpart of [`crate::ArchiveUnpacker`] for a world rather than a volume set. An installation answers for a
/// path with whichever copy wins — a loose file in `gamedata`, or an entry of the volume that file is standing in
/// front of — so extracting from one writes what the engine would load, not what any one volume holds.
///
/// Shadowed copies are never written. The destination is a tree with one file per engine path, which is what the
/// winner already is; writing the hidden copies too would produce a tree the engine could not have loaded, and there
/// is nowhere to put the second one.
///
/// Directories are not recreated for their own sake. A world has no directory records — the mounts answer with files
/// — so a directory exists in the output exactly when something was written into it.
pub struct XrayWorldExtractor;

impl XrayWorldExtractor {
  /// Write every entry the world resolves under one directory into a destination root.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when nothing resolves under the prefix, and whatever reading or writing an entry
  /// answers with.
  pub fn extract_directory<P: AsRef<Path>>(
    probe: &XrayProbe,
    prefix: &str,
    destination: P,
  ) -> XrfResult<ArchiveExtractDirectoryResult> {
    Self::extract_directory_opt(probe, prefix, destination, ArchiveExtractOptions::default())
  }

  /// Write every entry the world resolves under one directory into a destination root, as `options` asks.
  ///
  /// Keeps the layout below the prefix but not the prefix itself, exactly as an archive extraction does: extracting
  /// `configs\gameplay` into `C:\out` produces `C:\out\dialogs.xml`. An empty prefix means the whole world, which is
  /// what selecting the tree root does.
  ///
  /// What lies below the prefix is decided by the mounts rather than by the caller, so it is written through a
  /// [`RootedDestination`] rather than joined and opened.
  ///
  /// Cancellation lands between entries and keeps what it wrote, the same way an unpack does: the destination may
  /// hold the caller's own files, and nothing here can tell those from this run's.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when nothing resolves under the prefix, and whatever reading or writing an entry
  /// answers with.
  pub fn extract_directory_opt<P: AsRef<Path>>(
    probe: &XrayProbe,
    prefix: &str,
    destination: P,
    options: ArchiveExtractOptions,
  ) -> XrfResult<ArchiveExtractDirectoryResult> {
    let normalized: String = prefix.trim_end_matches(['\\', '/']).to_string();
    let destination: RootedDestination = RootedDestination::new(destination.as_ref());
    let job = &options.job;

    destination.create_root()?;

    // Selected before anything is written, so the run knows how much it is about to do — the same reason an archive
    // extraction selects first. The world is listed rather than filtered per read, because a probe answers for a
    // whole mounted order and asking it once is what makes the total knowable.
    let selected: Vec<(XrayMountedEntry, PathBuf)> = probe
      .list_mounted_entries()
      .into_iter()
      .filter_map(|entry| {
        relative_to_prefix(entry.get_logical_path(), &normalized)
          .map(ToOwned::to_owned)
          .map(|relative| (entry, relative))
      })
      .map(|(entry, relative)| to_host_relative(&relative).map(|relative| (entry, relative)))
      .collect::<XrfResult<Vec<(XrayMountedEntry, PathBuf)>>>()?;

    if selected.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "Cannot extract '{normalized}' - nothing in this installation resolves under it."
      )));
    }

    let mut extracted_count: usize = 0;
    let mut size: u64 = 0;
    let mut outcome: JobOutcome = JobOutcome::Completed;

    let extracting: JobScope = job.enter(EXTRACT_PHASE_WRITE, Some(selected.len() as u64));

    for (entry, relative) in &selected {
      // Before the write rather than after it: a payload already being written cannot be halved without leaving a
      // truncated file indistinguishable from a short one.
      if job.is_cancelled() {
        outcome = JobOutcome::Cancelled;

        break;
      }

      let bytes: Vec<u8> = probe.read_asset_bytes(&entry.asset)?;

      destination.create_file(relative)?.write_all(&bytes)?;

      extracted_count += 1;
      size += bytes.len() as u64;

      extracting.advance();
    }

    Ok(ArchiveExtractDirectoryResult {
      prefix: normalized,
      destination: format_path(destination.get_root()).to_string(),
      outcome,
      extracted_count,
      size,
    })
  }

  /// Write the copy the world resolves for one engine path to an exact path of the caller's choosing.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when nothing resolves the path, and whatever reading or writing it answers with.
  pub fn extract_file<P: AsRef<Path>>(
    probe: &XrayProbe,
    name: &str,
    destination: P,
  ) -> XrfResult<ArchiveExtractResult> {
    let asset: XrayAsset = probe
      .find(name)?
      .get_asset()
      .cloned()
      .ok_or_else(|| XrfError::new_not_found_error(format!("Cannot extract '{name}' - nothing here resolves it.")))?;

    let bytes: Vec<u8> = probe.read_asset_bytes(&asset)?;

    // The caller named this path, so it is written where it points rather than through a rooted destination — the
    // same rule an archive extraction of one file follows.
    if let Some(parent) = destination.as_ref().parent() {
      fs::create_dir_all(parent)?;
    }

    fs::write(destination.as_ref(), &bytes).map_err(|error| {
      XrfError::new_io_error(
        format!("Failed to write '{}': {error}", format_path(destination.as_ref())),
        error.kind(),
      )
    })?;

    Ok(ArchiveExtractResult {
      name: asset.get_logical_path().as_str().to_string(),
      destination: format_path(destination.as_ref()).to_string(),
      size: bytes.len() as u64,
    })
  }
}
