/// How one initialization run reports.
///
/// Carries no path: what to initialize is the operation's argument. The path used to sit here as well as being passed
/// separately, and the copy here was never read.
#[derive(Default)]
pub struct TranslationInitializeOptions {
  pub output: xrf_output::OutputOptions,
}

impl TranslationInitializeOptions {
  /// Send human output to `output`.
  #[must_use]
  pub fn with_output(mut self, output: xrf_output::OutputOptions) -> Self {
    self.output = output;
    self
  }
}
