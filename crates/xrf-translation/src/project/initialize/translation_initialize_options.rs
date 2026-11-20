use crate::project::events::translation_listener::TranslationListener;

/// How one initialization run reports.
///
/// Carries no path: what to initialize is the operation's argument. The path used to sit here as well as being passed
/// separately, and the copy here was never read.
#[derive(Default)]
pub struct TranslationInitializeOptions {
  /// What hears the run's events as they happen, or nothing.
  pub listener: Option<TranslationListener>,
}

impl TranslationInitializeOptions {
  /// Hand the run's events to `listener`.
  #[must_use]
  pub fn with_listener(mut self, listener: TranslationListener) -> Self {
    self.listener = Some(listener);
    self
  }
}
