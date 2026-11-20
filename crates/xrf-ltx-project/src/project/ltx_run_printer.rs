use std::path::Path;
use std::sync::Arc;

use xrf_output::{OutputChannel, OutputOptions};
use xrf_utils::{format_duration, format_path};

use crate::project::{LtxProject, LtxProjectFormatResult, LtxProjectVerifyResult, LtxVerifyEvent, LtxVerifyListener};

/// What a person reads of a verification or formatting run: the lines every surface running one prints, kept in one
/// place so `ltx verify`, `ltx format` and the gamedata LTX check say the same thing about the same run.
///
/// The runs themselves print nothing and answer results; this renders those results.
pub struct LtxRunPrinter;

impl LtxRunPrinter {
  /// Names the path a verification is about to judge, before it starts.
  pub fn print_verify_start(project: &LtxProject, output: &OutputOptions) {
    xrf_output::heading!(output, "Verify path: {}", format_path(&project.root));
  }

  /// A listener printing each field a verification checks, or none when verbose lines would not show, so a run nobody
  /// is reading resolves no paths for it.
  pub fn new_listener(output: &OutputOptions) -> Option<LtxVerifyListener> {
    if !output.is_visible(OutputChannel::Verbose) {
      return None;
    }

    let output: OutputOptions = output.clone();

    Some(Arc::new(move |event: &LtxVerifyEvent<'_>| {
      Self::print_event(event, &output);
    }))
  }

  /// Says one event at verbose level.
  pub fn print_event(event: &LtxVerifyEvent<'_>, output: &OutputOptions) {
    match *event {
      LtxVerifyEvent::FieldChecked {
        path, section, field, ..
      } => xrf_output::verbose!(output, "Checking {} [{section}] {field}", format_path(path)),
    }
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
  pub fn print_verify_result(project: &LtxProject, result: &LtxProjectVerifyResult, output: &OutputOptions) {
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
