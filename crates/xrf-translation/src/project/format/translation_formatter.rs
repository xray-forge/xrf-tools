use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_job::{JobOutcome, JobScope};
use xrf_utils::format_path;

use crate::json::translation_json_canon::TranslationJsonCanon;
use crate::json::translation_json_reader::TranslationJsonReader;
use crate::json::translation_json_render::TranslationJsonRender;
use crate::project::events::translation_emitter::TranslationEmitter;
use crate::project::events::translation_event::TranslationEvent;
use crate::project::format::translation_format_options::TranslationFormatOptions;
use crate::project::format::translation_format_result::TranslationFormatResult;
use crate::project::format::translation_source_selection::TranslationSourceSelection;
use crate::text::translation_json::TranslationJson;

/// Normalizes multi-language JSON translation sources in place.
pub struct TranslationFormatter;

impl TranslationFormatter {
  /// Phase a formatting run reports while it normalizes sources in place.
  pub const PHASE: &str = "format";

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
    let files: Vec<PathBuf> = TranslationSourceSelection::select(paths)?;

    result.startup_duration = options.job.elapsed();

    TranslationEmitter::of(options.listener.as_ref()).emit(TranslationEvent::FormatStarted {
      files: files.len(),
      is_check,
    });

    let formatting: JobScope = options.job.enter(Self::PHASE, Some(files.len() as u64));

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

    Ok(result)
  }

  /// Judge one source, and rewrite it unless this is a check.
  fn format_source(
    path: &Path,
    options: &TranslationFormatOptions,
    is_check: bool,
    result: &mut TranslationFormatResult,
  ) -> XrfResult {
    let mut parsed: TranslationJson = TranslationJsonReader::read(path)?;

    TranslationJsonCanon::sort(&mut parsed);

    let render: TranslationJsonRender = TranslationJsonRender::render(path, &parsed, options.line_endings)?;
    let is_changed: bool = render.is_changed();

    result.record(path.to_path_buf(), is_changed);

    // A rewrite is said once it is on disk, so a run that fails writing it has not claimed it.
    if is_changed && !is_check {
      render.write(path)?;
    }

    TranslationEmitter::of(options.listener.as_ref()).emit(TranslationEvent::SourceFormatted {
      path,
      is_changed,
      is_check,
    });

    Ok(())
  }
}
