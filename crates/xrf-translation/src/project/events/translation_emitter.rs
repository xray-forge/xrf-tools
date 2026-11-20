use crate::project::events::translation_event::TranslationEvent;
use crate::project::events::translation_listener::TranslationListener;

/// Hands a run's events to the listener its options carry, or does nothing at all when they carry none.
pub(crate) struct TranslationEmitter<'l> {
  listener: Option<&'l TranslationListener>,
}

impl<'l> TranslationEmitter<'l> {
  /// Called as `of(options.listener.as_ref())`.
  pub(crate) fn of(listener: Option<&'l TranslationListener>) -> Self {
    Self { listener }
  }

  /// Whether anything hears what is emitted here, for an event that costs work to build.
  pub(crate) const fn is_listening(&self) -> bool {
    self.listener.is_some()
  }

  pub(crate) fn emit(&self, event: TranslationEvent<'_>) {
    if let Some(listener) = self.listener {
      listener(&event);
    }
  }
}
