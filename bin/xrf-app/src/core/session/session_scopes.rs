use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::core::session::session_scopes_state::SessionScopesState;
use crate::core::session::{SessionId, SessionRelease, SessionScope};

/// The scope each opening is bound to, and every session that holds openings, so releasing a scope closes what it held
/// in every domain at once and no domain listens for it. A clone answers for the same bindings.
///
/// Only an explicit ending releases: a tool left, a window closed, or a page reporting which of its window's scopes are
/// still live. A page load releases nothing, since the openings outlive the page that asked for them.
#[derive(Clone, Default)]
pub(crate) struct SessionScopes {
  state: Arc<Mutex<SessionScopesState>>,
}

impl SessionScopes {
  /// Takes `session` as one a released scope closes openings in; a session registers itself as it is made.
  pub fn register(&self, session: impl SessionRelease + 'static) {
    self.lock().sessions.push(Arc::new(session));
  }

  /// Binds an opening to the scope that asked for it, before the open, so a scope released while it opens closes it.
  pub fn bind(&self, id: SessionId, scope: SessionScope) {
    self.lock().bindings.insert(id, scope);
  }

  /// The openings bound to `scope`, in no particular order.
  pub fn list(&self, scope: &SessionScope) -> Vec<SessionId> {
    self
      .lock()
      .bindings
      .iter()
      .filter(|(_, bound)| *bound == scope)
      .map(|(id, _)| *id)
      .collect()
  }

  /// Closes every opening bound to `scope`, answering how many were.
  pub fn release(&self, scope: &SessionScope) -> usize {
    self.release_where(|bound| bound == scope)
  }

  /// Closes every opening of `window` bound to a scope other than those in `live`, answering how many were: what a page
  /// left behind it on a load, which no tool of the page holds any longer.
  pub fn retain(&self, window: &str, live: &[SessionScope]) -> usize {
    self.release_where(|bound| bound.window == window && !live.contains(bound))
  }

  /// Closes every opening of a window that went, answering how many were.
  pub fn release_window(&self, window: &str) -> usize {
    self.release_where(|bound| bound.window == window)
  }

  fn release_where(&self, is_released: impl Fn(&SessionScope) -> bool) -> usize {
    let mut state: MutexGuard<'_, SessionScopesState> = self.lock();
    let mut ids: Vec<SessionId> = Vec::new();

    state.bindings.retain(|id, bound| {
      let is_kept: bool = !is_released(bound);

      if !is_kept {
        ids.push(*id);
      }

      is_kept
    });

    let sessions: Vec<Arc<dyn SessionRelease>> = state.sessions.clone();

    // Closed outside the lock: a session drops what it held, which may take a while for a large one.
    drop(state);

    if !ids.is_empty() {
      for session in &sessions {
        session.release(&ids);
      }
    }

    ids.len()
  }

  /// The bindings, whatever a panicking caller left: an entry is only ever added or removed whole.
  fn lock(&self) -> MutexGuard<'_, SessionScopesState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
