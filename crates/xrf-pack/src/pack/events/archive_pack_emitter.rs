use crate::pack::events::ArchivePackEvent;

/// Hands a run's events to its listener, or does nothing at all when it has none.
///
/// [`Self::is_listening`] answers once for a caller whose cost is the work it would do to produce an event, such as
/// recording what the walk left out, rather than the event itself. Emitting changes no byte of what is written.
pub(crate) struct ArchivePackEmitter<'l> {
  listener: Option<&'l dyn Fn(&ArchivePackEvent<'_>)>,
}

impl<'l> ArchivePackEmitter<'l> {
  pub(crate) fn new(listener: Option<&'l dyn Fn(&ArchivePackEvent<'_>)>) -> Self {
    Self { listener }
  }

  /// Whether anything hears what is emitted here.
  pub(crate) const fn is_listening(&self) -> bool {
    self.listener.is_some()
  }

  pub(crate) fn emit(&self, event: ArchivePackEvent<'_>) {
    if let Some(listener) = self.listener {
      listener(&event);
    }
  }
}
