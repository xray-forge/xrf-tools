use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_utils::format_path;

use crate::json::normalize::sort_document;
use crate::json::read::read_json;
use crate::json::write::{CanonicalRender, render_canonical, write_canonical};
use crate::project::format::options::ProjectFormatOptions;
use crate::project::format::result::ProjectFormatResult;
use crate::project::format::selection::select_sources;
use crate::types::TranslationJson;

/// Normalize translation sources in place, or report which ones are not normalized.
///
/// The check and the rewrite are the same walk with the write suppressed, so a check that passes and a rewrite that
/// changes something cannot disagree about a file.
///
/// # Errors
///
/// Returns a not-found error for a path that does not exist, an invalid error when the paths select nothing, a parsing
/// error for a source that cannot be read, and an IO error when one cannot be replaced. A malformed source stops the
/// run rather than becoming a finding: a formatter has nothing to write except what it read, and `parse` refuses to
/// overwrite an unparsable target for the same reason.
pub fn format_sources(options: &ProjectFormatOptions) -> XrfResult<ProjectFormatResult> {
  let mut result: ProjectFormatResult = ProjectFormatResult::new();

  // Selected inside the measured span, because walking a large tree before the first file is read is part of the wait
  // the caller is asking about.
  let files: Vec<PathBuf> = select_sources(&options.paths)?;

  result.startup_duration = options.job.elapsed();

  xrf_output::heading!(
    options.output,
    "{} {} translation source(s)",
    if options.is_check { "Checking" } else { "Formatting" },
    files.len()
  );

  let formatting: xrf_job::JobScope = options.job.enter(
    crate::project::job_phases::TRANSLATION_PHASE_FORMAT,
    Some(files.len() as u64),
  );

  for file in &files {
    // Between files, never inside one. A stopped check reports what it had judged and says so; findings read as a
    // complete verdict otherwise.
    if options.job.is_cancelled() {
      result.outcome = xrf_job::JobOutcome::Cancelled;

      break;
    }

    // Sequential, so naming the file being rewritten is meaningful rather than whichever one a worker happens to hold.
    options.job.set_detail(Some(format_path(file).to_string()));

    format_source(file, options, &mut result)?;

    formatting.advance();
  }

  options.job.set_detail(None);

  result.duration = options.job.elapsed();

  report(&result, options);

  Ok(result)
}

/// Judge one source, and rewrite it unless this is a check.
fn format_source(path: &Path, options: &ProjectFormatOptions, result: &mut ProjectFormatResult) -> XrfResult {
  let mut parsed: TranslationJson = read_json(path)?;

  sort_document(&mut parsed);

  let render: CanonicalRender = render_canonical(path, &parsed, options.line_endings)?;
  let is_changed: bool = render.is_changed();

  result.record(path.to_path_buf(), is_changed);

  if !is_changed {
    xrf_output::verbose!(options.output, "Formatted: {}", format_path(path));

    return Ok(());
  }

  if options.is_check {
    xrf_output::info!(options.output, "Not formatted: {}", format_path(path));

    return Ok(());
  }

  write_canonical(path, &render)?;

  xrf_output::info!(options.output, "Formatted: {}", format_path(path));

  Ok(())
}

/// Report the totals a run finished with.
///
/// The two modes report separately because they mean opposite things by the same number. A check states a verdict, and
/// a clean one is a success worth saying; a rewrite states work done, and saying a tree was "checked" would describe a
/// run that wrote to it.
fn report(result: &ProjectFormatResult, options: &ProjectFormatOptions) {
  let duration: String = xrf_utils::format_duration(result.duration);

  if !options.is_check {
    xrf_output::info!(
      options.output,
      "Formatted {}/{} translation source(s) in {duration}",
      result.invalid_files,
      result.total_files
    );

    return;
  }

  if result.invalid_files == 0 {
    xrf_output::success!(
      options.output,
      "All {} translation source(s) are formatted, checked in {duration}",
      result.total_files
    );
  } else {
    xrf_output::warning!(
      options.output,
      "Format issues with {}/{} translation source(s) in {duration}",
      result.invalid_files,
      result.total_files
    );
  }
}
