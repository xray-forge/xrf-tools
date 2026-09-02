//! Rewrites every visual under a path into well-formed bytes and accounts for what went.
//!
//! Lives in the CLI rather than in `xrf-db` because everything here is the command's own: the walk over a host
//! directory, the staged write that keeps a previous file whole, and the report a caller reads back. What a well-formed
//! visual *is* stays with the crate — `OgfNormalization` produces the bytes and proves the engine reads them identically.

use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};
use xrf_db::{OgfNormalization, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_output::{OutputOptions, OutputSequence, OutputSlot};
use xrf_utils::format_path;

use crate::commands::ogf::fix::report::{OgfFixFileReport, OgfFixFindingReport, OgfFixOutcome, OgfFixReport};
use crate::core::staged_write::write_file_staged;

const OGF_EXTENSION: &str = "ogf";

pub struct OgfFixer<'a> {
  output: &'a OutputOptions,
  is_dry_run: bool,
}

impl<'a> OgfFixer<'a> {
  pub fn new(output: &'a OutputOptions, is_dry_run: bool) -> Self {
    Self { output, is_dry_run }
  }

  /// Fix the visual at `path`, or every visual under it, into `destination` or in place.
  ///
  /// A sweep fixes what it can and reports what it could not; deciding that a refusal fails the run is the caller's.
  ///
  /// # Errors
  ///
  /// Returns an error before touching anything when the selection itself is wrong: a path that does not exist, a
  /// directory holding no visuals, or a destination given for a directory.
  pub fn fix(&self, path: &Path, destination: Option<&Path>) -> XrfResult<OgfFixReport> {
    if destination.is_some() && path.is_dir() {
      return Err(XrfError::new_invalid_error(format!(
        "Destination applies to a single ogf file, {} is a directory",
        format_path(path)
      )));
    }

    let files: Vec<PathBuf> = Self::list_visuals(path)?;

    self.report_selection(path, &files);

    let outcomes: Vec<OgfFixOutcome> = match files.as_slice() {
      [file] => vec![self.fix_visual(self.output, file, destination.unwrap_or(file))],
      files => self.fix_visuals(files),
    };

    Ok(OgfFixReport::new(outcomes, self.is_dry_run))
  }

  /// The visuals to act on: the file named, or every `.ogf` under the directory named, in path order.
  fn list_visuals(path: &Path) -> XrfResult<Vec<PathBuf>> {
    if path.is_file() {
      return Ok(vec![path.to_path_buf()]);
    }

    if !path.is_dir() {
      return Err(XrfError::new_not_found_error(format!(
        "OGF path was not found: {}",
        format_path(path)
      )));
    }

    let files: Vec<PathBuf> = WalkDir::new(path)
      .sort_by_file_name()
      .into_iter()
      .filter_map(Result::ok)
      .map(DirEntry::into_path)
      .filter(|it| {
        it.is_file()
          && it
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case(OGF_EXTENSION))
      })
      .collect();

    // A directory holding no visuals is refused rather than swept successfully: a mistyped path would otherwise report
    // a clean run over nothing.
    if files.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "No ogf visuals found in {}",
        format_path(path)
      )));
    }

    Ok(files)
  }

  fn report_selection(&self, path: &Path, files: &[PathBuf]) {
    let verb: &str = if self.is_dry_run { "Checking" } else { "Fixing" };

    match files {
      [file] => xrf_output::info!(self.output, "{verb} ogf visual {}", format_path(file)),
      files => xrf_output::info!(
        self.output,
        "{verb} {} ogf visuals in {}",
        files.len(),
        format_path(path)
      ),
    }
  }

  /// A sweep fans out and releases console output in path order, whatever order the workers finish in.
  fn fix_visuals(&self, files: &[PathBuf]) -> Vec<OgfFixOutcome> {
    let sequence: OutputSequence = OutputSequence::new(self.output, files.len());

    files
      .par_iter()
      .enumerate()
      .map(|(index, file)| {
        let slot: OutputSlot = sequence.new_slot(index);

        self.fix_visual(slot.get_output(), file, file)
      })
      .collect()
  }

  /// Normalize one visual: read, normalize, prove the engine still reads the same, say what goes, then write.
  ///
  /// With a destination of its own, the visual is written even when nothing changed, so the output exists either way.
  fn fix_visual(&self, output: &OutputOptions, source: &Path, destination: &Path) -> OgfFixOutcome {
    let original: Vec<u8> = match fs::read(source) {
      Ok(original) => original,
      Err(error) => return Self::fail(output, source, format!("File was not read: {error}")),
    };

    let normalization: OgfNormalization = match OgfNormalization::normalize::<XRayByteOrder>(&original) {
      Ok(normalization) => normalization,
      Err(error) => return Self::fail(output, source, error.to_string()),
    };

    let is_changed: bool = normalization.is_changed_from(&original);

    if !is_changed && destination == source {
      xrf_output::verbose!(output, "Well-formed: {}", format_path(source));

      return OgfFixOutcome::Unchanged;
    }

    if is_changed {
      if let Err(error) = normalization.assert_engine_reads_the_same::<XRayByteOrder>(&original) {
        return Self::fail(output, source, error.to_string());
      }

      Self::announce_discard(output, source, &original, &normalization);
    }

    if self.is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written, {} would receive {} bytes instead of {}",
        format_path(destination),
        normalization.bytes.len(),
        original.len()
      );
    } else {
      if let Err(error) = Self::write(destination, &normalization.bytes) {
        return Self::fail(output, source, error.to_string());
      }

      xrf_output::info!(output, "Ogf visual written into {}", format_path(destination));
    }

    if !is_changed {
      return OgfFixOutcome::Unchanged;
    }

    OgfFixOutcome::Normalized(OgfFixFileReport::new(
      source,
      destination,
      original.len(),
      normalization.bytes.len(),
      normalization.residue.as_ref(),
      !self.is_dry_run,
    ))
  }

  /// Say what the rewrite drops before it drops it, so the only record of a discarded path is not the file losing it.
  fn announce_discard(output: &OutputOptions, source: &Path, original: &[u8], normalization: &OgfNormalization) {
    xrf_output::info!(
      output,
      "Normalize {}: {} bytes the engine never reads",
      format_path(source),
      normalization.get_discarded_size(original)
    );

    if let Some(path) = normalization
      .residue
      .as_ref()
      .and_then(|residue| residue.cause.get_discarded_path())
    {
      xrf_output::warning!(
        output,
        "Discarding uncounted motion reference '{}' from {}",
        path,
        format_path(source)
      );
    }
  }

  /// Stage the bytes beside the destination and move them into place, so a failed write leaves the previous file whole.
  fn write(destination: &Path, bytes: &[u8]) -> XrfResult {
    if let Some(parent) = destination.parent()
      && !parent.as_os_str().is_empty()
    {
      fs::create_dir_all(parent)?;
    }

    write_file_staged(destination, bytes)
      .map_err(|error| XrfError::new_io_error(format!("File was not written: {error}"), error.kind()))
  }

  fn fail(output: &OutputOptions, source: &Path, message: String) -> OgfFixOutcome {
    xrf_output::error!(output, "Failed to fix {}: {}", format_path(source), message);

    OgfFixOutcome::Failed(OgfFixFindingReport::new(source, message))
  }
}
