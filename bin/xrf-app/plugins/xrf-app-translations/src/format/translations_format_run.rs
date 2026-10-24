use std::path::PathBuf;
use std::str::FromStr;

use xrf_app_core::TauriResult;
use xrf_job::JobHandle;
use xrf_translation::{TranslationFormatOptions, TranslationFormatResult, TranslationFormatter};
use xrf_utils::{LineEndings, error_to_string};

/// Normalizing the JSON translation sources under a directory, or checking that they are.
pub struct TranslationsFormatRun;

impl TranslationsFormatRun {
  /// Rewrites every source under `directory` that is not normalized, with `line_endings` where named.
  ///
  /// # Errors
  ///
  /// Returns an error when the line endings are not ones the formatter knows, or the walk fails.
  pub fn format(
    job: JobHandle,
    directory: PathBuf,
    line_endings: Option<String>,
  ) -> TauriResult<TranslationFormatResult> {
    TranslationFormatter::format_opt(&[directory], Self::to_options(job, line_endings)?).map_err(error_to_string)
  }

  /// Reports which sources under `directory` are not normalized, writing nothing.
  ///
  /// # Errors
  ///
  /// As [`Self::format`].
  pub fn check(
    job: JobHandle,
    directory: PathBuf,
    line_endings: Option<String>,
  ) -> TauriResult<TranslationFormatResult> {
    TranslationFormatter::check_format_opt(&[directory], Self::to_options(job, line_endings)?).map_err(error_to_string)
  }

  fn to_options(job: JobHandle, line_endings: Option<String>) -> TauriResult<TranslationFormatOptions> {
    let line_endings: Option<LineEndings> = line_endings
      .as_deref()
      .map(LineEndings::from_str)
      .transpose()
      .map_err(error_to_string)?;

    Ok(
      TranslationFormatOptions::default()
        .with_job(job)
        .with_line_endings(line_endings),
    )
  }
}
