use xrf_utils::LineEndings;

/// How one formatting run reports, is stopped, and spells its line endings.
///
/// What to format is the operation's argument rather than a field here, matching `ArchivePacker` and
/// `LtxFilesFormatter`: this carries how the run behaves, not what it acts on. Whether it writes is not here either -
/// that is the difference between [`TranslationFormatter::format_opt`] and [`TranslationFormatter::check_format_opt`],
/// and a boolean deciding whether a function rewrites a tree belongs in its name.
///
/// [`TranslationFormatter::format_opt`]: crate::TranslationFormatter::format_opt
/// [`TranslationFormatter::check_format_opt`]: crate::TranslationFormatter::check_format_opt
#[derive(Default)]
pub struct TranslationFormatOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub output: xrf_output::OutputOptions,
  /// Write these line endings rather than preserving each file's own.
  ///
  /// Also arms the check: with a choice made, a file spelling its endings the other way is not formatted, and the
  /// check says so. Without one, the comparison normalizes both sides and endings are left to `.gitattributes`.
  pub line_endings: Option<LineEndings>,
}

impl TranslationFormatOptions {
  /// Report and cancel through `job`.
  #[must_use]
  pub fn with_job(mut self, job: xrf_job::JobHandle) -> Self {
    self.job = job;
    self
  }

  /// Write and judge `line_endings` rather than preserving each file's own.
  #[must_use]
  pub fn with_line_endings(mut self, line_endings: Option<LineEndings>) -> Self {
    self.line_endings = line_endings;
    self
  }

  /// Send human output to `output`.
  #[must_use]
  pub fn with_output(mut self, output: xrf_output::OutputOptions) -> Self {
    self.output = output;
    self
  }
}
