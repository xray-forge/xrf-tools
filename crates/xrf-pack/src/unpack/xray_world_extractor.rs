use std::fs;
use std::io::Write;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{HostDestination, format_path};
use xrf_vfs::{XrayAsset, XrayMountedEntry, XrayProbe};

use crate::path::{relative_to_prefix, to_host_relative};
use crate::unpack::archive_extract_options::ArchiveExtractOptions;
use crate::unpack::archive_extract_result::{ArchiveExtractDirectoryResult, ArchiveExtractResult};
use crate::unpack::tree_entry::TreeEntry;
use crate::unpack::tree_write_summary::TreeWriteSummary;
use crate::unpack::tree_writer::TreeWriter;

/// Writes what a mounted world resolves out to a directory.
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
    let destination: HostDestination = HostDestination::new(destination.as_ref(), options.link_policy);

    destination.create_root()?;

    // Selected before anything is written, so the run knows how much it is about to do — the same reason an archive
    // extraction selects first. The world is listed rather than filtered per read, because a probe answers for a
    // whole mounted order and asking it once is what makes the total knowable.
    let selected: Vec<TreeEntry<XrayMountedEntry>> = probe
      .list_mounted_entries()
      .into_iter()
      .filter_map(|entry| {
        relative_to_prefix(entry.get_logical_path(), &normalized)
          .map(ToOwned::to_owned)
          .map(|relative| (entry, relative))
      })
      .map(|(entry, relative)| {
        Ok(TreeEntry {
          source: entry,
          relative: to_host_relative(&relative)?,
          is_directory: false,
        })
      })
      .collect::<XrfResult<Vec<TreeEntry<XrayMountedEntry>>>>()?;

    if selected.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "Cannot extract '{normalized}' - nothing in this installation resolves under it."
      )));
    }

    let summary: TreeWriteSummary = TreeWriter::write(&destination, &selected, &options.job, |entry, target| {
      let bytes: Vec<u8> = probe.read_asset_bytes(&entry.asset)?;

      target.write_all(&bytes)?;

      Ok(bytes.len() as u64)
    })?;

    Ok(ArchiveExtractDirectoryResult {
      prefix: normalized,
      destination: format_path(destination.get_root()).to_string(),
      outcome: summary.outcome,
      extracted_count: summary.files,
      size: summary.bytes,
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
