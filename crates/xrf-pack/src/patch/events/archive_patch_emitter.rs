use crate::patch::events::ArchivePatchEvent;

/// Hands a patch run's events to its listener, or does nothing at all when it has none.
pub(crate) struct ArchivePatchEmitter<'l> {
  listener: Option<&'l dyn Fn(&ArchivePatchEvent<'_>)>,
}

impl<'l> ArchivePatchEmitter<'l> {
  pub(crate) fn new(listener: Option<&'l dyn Fn(&ArchivePatchEvent<'_>)>) -> Self {
    Self { listener }
  }

  /// Whether anything hears what is emitted here.
  pub(crate) const fn is_listening(&self) -> bool {
    self.listener.is_some()
  }

  pub(crate) fn emit(&self, event: ArchivePatchEvent<'_>) {
    if let Some(listener) = self.listener {
      listener(&event);
    }
  }
}
