use crate::core::session::SessionId;

/// What holds openings a released scope closes: a domain's sessions.
pub(crate) trait SessionRelease: Send + Sync {
  /// Closes whichever of `ids` it holds, committed or pending; one it never held is no concern of it.
  fn release(&self, ids: &[SessionId]);
}
