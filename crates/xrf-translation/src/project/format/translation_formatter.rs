use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_job::{JobOutcome, JobScope};
use xrf_utils::format_path;

use crate::json::normalize::sort_document;
use crate::json::read::read_json;
use crate::json::write::{CanonicalRender, render_canonical, write_canonical};
use crate::project::format::translation_format_options::TranslationFormatOptions;
use crate::project::format::translation_format_result::TranslationFormatResult;
use crate::project::format::translation_source_selection::select_sources;
use crate::project::job_phases::TRANSLATION_PHASE_FORMAT;
use crate::types::TranslationJson;

/// Normalizes multi-language JSON translation sources in place.
pub struct TranslationFormatter;

impl TranslationFormatter {
  /// Rewrite every source under `paths` that is not canonical.
  ///
  /// The plain door. A caller that wants to watch the run, stop it, or assert its line endings uses
  /// [`Self::format_opt`].
  ///
  /// # Errors
  ///
  /// The same as [`Self::format_opt`].
  pub fn format(paths: &[PathBuf]) -> XrfResult<TranslationFormatResult> {
    Self::format_opt(paths, TranslationFormatOptions::default())
  }

  /// Report which sources under `paths` are not canonical, without writing anything.
  ///
  /// # Errors
  ///
  /// The same as [`Self::check_format_opt`].
  pub fn check_format(paths: &[PathBuf]) -> XrfResult<TranslationFormatResult> {
    Self::check_format_opt(paths, TranslationFormatOptions::default())
  }

  /// Rewrite every source under `paths` that is not canonical, reporting to and stoppable through `options`.
  ///
  /// A source already holding the canonical bytes is not written at all, so a run over a clean tree touches no mtimes.
  /// Each rewrite replaces the file whole through a staged write, so a run that fails or is stopped leaves the sources
  /// it had already normalized normalized and the rest untouched — never one half-written.
  ///
  /// # Errors
  ///
  /// Returns a not-found error for a path that does not exist, an invalid error when the paths select nothing, a
  /// parsing error for a source that cannot be read, and an IO error when one cannot be replaced.
  pub fn format_opt(paths: &[PathBuf], options: TranslationFormatOptions) -> XrfResult<TranslationFormatResult> {
    Self::run(paths, &options, false)
  }

  /// Report which sources under `paths` are not canonical, reporting to and stoppable through `options`.
  ///
  /// The same walk as [`Self::format_opt`] with the write suppressed, which is what makes the two agree: a check that
  /// passes and a rewrite that changes something cannot disagree about a file, because one comparison answers both.
  ///
  /// # Errors
  ///
  /// The same as [`Self::format_opt`], minus the write.
  pub fn check_format_opt(paths: &[PathBuf], options: TranslationFormatOptions) -> XrfResult<TranslationFormatResult> {
    Self::run(paths, &options, true)
  }

  /// The walk both modes share, with `is_check` deciding only whether the rewrite is published.
  fn run(paths: &[PathBuf], options: &TranslationFormatOptions, is_check: bool) -> XrfResult<TranslationFormatResult> {
    let mut result: TranslationFormatResult = TranslationFormatResult::new();

    // Selected inside the measured span, because walking a large tree before the first file is read is part of the
    // wait the caller is asking about.
    let files: Vec<PathBuf> = select_sources(paths)?;

    result.startup_duration = options.job.elapsed();

    xrf_output::heading!(
      options.output,
      "{} {} translation source(s)",
      if is_check { "Checking" } else { "Formatting" },
      files.len()
    );

    let formatting: JobScope = options.job.enter(TRANSLATION_PHASE_FORMAT, Some(files.len() as u64));

    for file in &files {
      // Between files, never inside one. A stopped check reports what it had judged and says so; findings read as a
      // complete verdict otherwise.
      if options.job.is_cancelled() {
        result.outcome = JobOutcome::Cancelled;

        break;
      }

      // Sequential, so naming the file being rewritten is meaningful rather than whichever one a worker happens to
      // hold.
      options.job.set_detail(Some(format_path(file).to_string()));

      Self::format_source(file, options, is_check, &mut result)?;

      formatting.advance();
    }

    options.job.set_detail(None);

    result.duration = options.job.elapsed();

    Self::report(&result, options, is_check);

    Ok(result)
  }

  /// Judge one source, and rewrite it unless this is a check.
  fn format_source(
    path: &Path,
    options: &TranslationFormatOptions,
    is_check: bool,
    result: &mut TranslationFormatResult,
  ) -> XrfResult {
    let mut parsed: TranslationJson = read_json(path)?;

    sort_document(&mut parsed);

    let render: CanonicalRender = render_canonical(path, &parsed, options.line_endings)?;
    let is_changed: bool = render.is_changed();

    result.record(path.to_path_buf(), is_changed);

    if !is_changed {
      xrf_output::verbose!(options.output, "Formatted: {}", format_path(path));

      return Ok(());
    }

    if is_check {
      xrf_output::info!(options.output, "Not formatted: {}", format_path(path));

      return Ok(());
    }

    write_canonical(path, &render)?;

    xrf_output::info!(options.output, "Formatted: {}", format_path(path));

    Ok(())
  }

  /// Report the totals a run finished with.
  ///
  /// The two modes report separately because they mean opposite things by the same number. A check states a verdict,
  /// and a clean one is a success worth saying; a rewrite states work done, and saying a tree was "checked" would
  /// describe a run that wrote to it.
  fn report(result: &TranslationFormatResult, options: &TranslationFormatOptions, is_check: bool) {
    let duration: String = xrf_utils::format_duration(result.duration);

    if !is_check {
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
}
