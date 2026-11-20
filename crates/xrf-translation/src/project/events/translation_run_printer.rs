use xrf_output::OutputOptions;
use xrf_utils::{format_duration, format_path};

use crate::project::events::translation_event::TranslationEvent;
use crate::project::events::translation_listener::TranslationListener;
use crate::project::format::translation_format_result::TranslationFormatResult;

/// What a person reads of a translation run: each event as it happens and a format run's totals, kept in one place so
/// every surface running one says the same thing about the same run.
///
/// The runs themselves print nothing; this renders what they hand out and answer.
pub struct TranslationRunPrinter;

impl TranslationRunPrinter {
  /// A listener printing every event to `output`.
  pub fn new_listener(output: &OutputOptions) -> TranslationListener {
    let output: OutputOptions = output.clone();

    Box::new(move |event: &TranslationEvent<'_>| Self::print_event(event, &output))
  }

  /// Says one event at the level it is worth.
  pub fn print_event(event: &TranslationEvent<'_>, output: &OutputOptions) {
    match *event {
      TranslationEvent::FileSkipped { path } => xrf_output::info!(output, "Skip file {}", format_path(path)),
      TranslationEvent::BuildStarted { sources } => {
        xrf_output::info!(output, "Building {sources} translation source(s)");
      }
      TranslationEvent::SourceBuilding { subject } => xrf_output::verbose!(output, "Building translations {subject}"),
      TranslationEvent::TableWriting { path } => xrf_output::verbose!(output, "Writing file {}", format_path(path)),
      TranslationEvent::FormatStarted { files, is_check } => xrf_output::heading!(
        output,
        "{} {files} translation source(s)",
        if is_check { "Checking" } else { "Formatting" }
      ),
      TranslationEvent::SourceFormatted {
        path,
        is_changed,
        is_check,
      } => match (is_changed, is_check) {
        (false, _) => xrf_output::verbose!(output, "Formatted: {}", format_path(path)),
        (true, true) => xrf_output::info!(output, "Not formatted: {}", format_path(path)),
        (true, false) => xrf_output::info!(output, "Formatted: {}", format_path(path)),
      },
      TranslationEvent::InitializeStarted { dir } => xrf_output::info!(output, "Initializing dir {}", format_path(dir)),
      TranslationEvent::KeyInitialized { id, language } => {
        xrf_output::info!(output, "Initializing missing key: {id} - {language}");
      }
      TranslationEvent::ParseStarted {
        scope,
        language,
        is_dry_run,
      } => xrf_output::info!(
        output,
        "Parsing translations at {scope} as '{language}'{}",
        if is_dry_run { " (dry run)" } else { "" }
      ),
      TranslationEvent::SourceUnchanged { path } => xrf_output::verbose!(output, "Unchanged {}", format_path(path)),
      TranslationEvent::SourceWritten {
        path,
        is_new,
        is_dry_run,
      } => xrf_output::info!(
        output,
        "{} {}",
        if is_dry_run {
          "Would write"
        } else if is_new {
          "Creating"
        } else {
          "Updating"
        },
        format_path(path)
      ),
      TranslationEvent::VerifyStarted { sources } => {
        xrf_output::info!(output, "Verifying {sources} translation source(s)");
      }
      TranslationEvent::KeyMissing { key, language, subject } => {
        xrf_output::error!(output, "Translation key missing: {key} {language} in {subject}");
      }
      TranslationEvent::ProblemFound { subject, problem } => {
        xrf_output::error!(output, "{subject}: {}", problem.message);
      }
    }
  }

  /// The totals a format run or check finished with.
  ///
  /// The two modes report separately because they mean opposite things by the same number. A check states a verdict,
  /// and a clean one is a success worth saying; a rewrite states work done, and saying a tree was "checked" would
  /// describe a run that wrote to it.
  pub fn print_format_result(result: &TranslationFormatResult, is_check: bool, output: &OutputOptions) {
    let duration: String = format_duration(result.duration);

    if !is_check {
      xrf_output::info!(
        output,
        "Formatted {}/{} translation source(s) in {duration}",
        result.invalid_files,
        result.total_files
      );
    } else if result.invalid_files == 0 {
      xrf_output::success!(
        output,
        "All {} translation source(s) are formatted, checked in {duration}",
        result.total_files
      );
    } else {
      xrf_output::warning!(
        output,
        "Format issues with {}/{} translation source(s) in {duration}",
        result.invalid_files,
        result.total_files
      );
    }
  }
}
