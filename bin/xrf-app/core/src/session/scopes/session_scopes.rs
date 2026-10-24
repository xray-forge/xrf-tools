use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rayon::ThreadPool;

use crate::session::scopes::session_bindings::SessionBindings;
use crate::session::{SessionId, SessionRelease, SessionScope};

/// The scope each opening is bound to, and every session that holds openings, so releasing a scope closes what it held
/// in every domain at once and no domain listens for it. A clone answers for the same bindings.
///
/// Only an explicit ending releases: a tool left, a window closed, or a page reporting which of its window's scopes are
/// still live. A page load releases nothing, since the openings outlive the page that asked for them. An opening is
/// bound before it opens, and a session refuses to open one that is not, so every handle is one a release closes.
#[derive(Clone, Default)]
pub struct SessionScopes {
  bindings: SessionBindings,
  sessions: Arc<Mutex<Vec<Arc<dyn SessionRelease>>>>,
}

impl SessionScopes {
  /// The scopes of an application whose closed documents are dropped on `pool`: a whole level takes a while to drop,
  /// which neither a command nor a window event should wait for. Without one, as in tests, they drop where they close.
  pub fn new(pool: Arc<ThreadPool>) -> Self {
    Self {
      bindings: SessionBindings::new(pool),
      sessions: Arc::default(),
    }
  }

  /// Takes `session` as one a released scope closes openings in; a session registers itself as it is made.
  pub(crate) fn register(&self, session: impl SessionRelease + 'static) {
    self.lock().push(Arc::new(session));
  }

  /// The bindings a session opens and closes through, which hold no session in turn.
  pub(crate) fn get_bindings(&self) -> SessionBindings {
    self.bindings.clone()
  }

  /// Binds an opening to the scope that asked for it, before the open, so a scope released while it opens closes it.
  pub fn bind(&self, id: SessionId, scope: SessionScope) {
    self.bindings.bind(id, scope);
  }

  /// The openings bound to `scope`, in no particular order.
  pub fn list(&self, scope: &SessionScope) -> Vec<SessionId> {
    self.bindings.list(scope)
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

  /// Unbinds the openings `is_released` names, then closes them in every session; what they held is dropped on the
  /// pool, so a release takes no longer than its bookkeeping. Unbound first, so an open that reserves its handle after
  /// the unbinding finds it so and gives up.
  fn release_where(&self, is_released: impl Fn(&SessionScope) -> bool) -> usize {
    let ids: Vec<SessionId> = self.bindings.take_where(is_released);

    if !ids.is_empty() {
      // Copied out, so a session closing a handle runs under no lock of the registry's.
      let sessions: Vec<Arc<dyn SessionRelease>> = self.lock().clone();

      for session in &sessions {
        session.release(&ids);
      }
    }

    ids.len()
  }

  /// The registry, whatever a panicking caller left: a session is only ever added whole.
  fn lock(&self) -> MutexGuard<'_, Vec<Arc<dyn SessionRelease>>> {
    self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
