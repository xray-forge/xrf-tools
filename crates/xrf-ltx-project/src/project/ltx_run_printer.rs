use std::path::Path;

use xrf_error::XrfResult;
use xrf_output::{OutputChannel, OutputOptions};
use xrf_utils::{format_duration, format_path};
use xrf_vfs::XrayLogicalPath;

use crate::project::{LtxProject, LtxProjectFormatResult, LtxProjectVerifyResult, LtxVerifyOptions};

/// What a person reads of a verification or formatting run: the lines every surface running one prints, kept in one
/// place so `ltx verify`, `ltx format` and the gamedata LTX check say the same thing about the same run.
///
/// The runs themselves print nothing and answer results; this renders those results.
pub struct LtxRunPrinter;

impl LtxRunPrinter {
  /// Verifies `project` as [`LtxProject::verify_entries_opt`] does, printing the run around it: the path it verifies,
  /// each field it checks when the output is verbose, then each finding and the totals.
  ///
  /// # Errors
  ///
  /// As [`LtxProject::verify_entries_opt`].
  pub fn verify(
    project: &LtxProject,
    options: LtxVerifyOptions,
    output: &OutputOptions,
  ) -> XrfResult<LtxProjectVerifyResult> {
    xrf_output::heading!(output, "Verify path: {}", format_path(&project.root));

    let result: LtxProjectVerifyResult = if output.is_visible(OutputChannel::Verbose) {
      // Rendered once per entry point rather than per field: rendering asks the VFS where the file sits.
      let mut reported: Option<(XrayLogicalPath, String)> = None;

      project.verify_entries_traced(options, |entry, section, field| {
        if reported.as_ref().is_none_or(|(rendered, _)| rendered != entry) {
          reported = Some((entry.clone(), format_path(&project.path_of(entry)).to_string()));
        }

        if let Some((_, path)) = &reported {
          xrf_output::verbose!(output, "Checking {path} [{section}] {field}");
        }
      })?
    } else {
      project.verify_entries_opt(options)?
    };

    Self::print_verify_result(project, &result, output);

    Ok(result)
  }

  /// Names the files a formatting run is about to read, before it starts.
  pub fn print_format_start(files: usize, is_check: bool, output: &OutputOptions) {
    if is_check {
      xrf_output::heading!(output, "Checking {files} file(s)");
    } else {
      xrf_output::heading!(output, "Formatting {files} file(s)");
    }
  }

  /// Names each file a run rewrote, or a check found unformatted, then the totals.
  pub fn print_format_result(result: &LtxProjectFormatResult, is_check: bool, output: &OutputOptions) {
    let duration: String = format_duration(result.duration);

    for file in &result.to_format {
      Self::print_format_file(file, is_check, output);
    }

    if !is_check {
      xrf_output::info!(
        output,
        "Formatted {}/{} files in {duration}",
        result.invalid_files,
        result.total_files
      );
    } else if result.invalid_files == 0 {
      xrf_output::success!(
        output,
        "All {} files are formatted, checked in {duration}",
        result.total_files
      );
    } else {
      xrf_output::warning!(
        output,
        "Format issues with {}/{} files in {duration}",
        result.invalid_files,
        result.total_files
      );
    }
  }

  fn print_format_file(file: &Path, is_check: bool, output: &OutputOptions) {
    if is_check {
      xrf_output::info!(output, "Not formatted: {}", format_path(file));
    } else {
      xrf_output::info!(output, "Formatted: {}", format_path(file));
    }
  }

  /// Each finding at the path a person opens, then how much of the project the run judged: the share of sections a
  /// scheme bound, which the findings alone do not show.
  fn print_verify_result(project: &LtxProject, result: &LtxProjectVerifyResult, output: &OutputOptions) {
    for finding in &result.findings {
      xrf_output::error!(output, "{}", project.with_reported_paths(finding));
    }

    xrf_output::info!(
      output,
      "Checked {} files, {} sections in {}",
      project.ltx_files.len(),
      result.total_sections,
      format_duration(result.duration)
    );

    if result.total_sections == 0 {
      xrf_output::info!(output, "No LTX sections were verified");
    } else {
      xrf_output::info!(
        output,
        "Verified {:.2}%, {} files, {} sections, {} fields",
        (result.checked_sections as f64 * 100.0) / result.total_sections as f64,
        result.total_files,
        result.checked_sections,
        result.checked_fields
      );
    }

    xrf_output::info!(output, "Found {} finding(s)", result.findings.len());
  }
}
