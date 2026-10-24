use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rayon::ThreadPool;

use crate::session::{SessionId, SessionScope};

/// The scope each opening is bound to, and the pool what a closed one held is dropped on: what a domain's sessions ask
/// of the scopes, apart from the registry of sessions a release closes openings in, so a session holds no registry that
/// holds it. A clone answers for the same bindings.
#[derive(Clone, Default)]
pub(crate) struct SessionBindings {
  bindings: Arc<Mutex<HashMap<SessionId, SessionScope>>>,
  /// Where a closed document is dropped, none dropping it where it closes.
  pool: Option<Arc<ThreadPool>>,
}

impl SessionBindings {
  /// Bindings whose closed documents are dropped on `pool`.
  pub fn new(pool: Arc<ThreadPool>) -> Self {
    Self {
      bindings: Arc::default(),
      pool: Some(pool),
    }
  }

  pub fn bind(&self, id: SessionId, scope: SessionScope) {
    self.lock().insert(id, scope);
  }

  pub fn is_bound(&self, id: SessionId) -> bool {
    self.lock().contains_key(&id)
  }

  /// Forgets the bindings of openings closed one by one, so a later release does not name them again.
  pub fn unbind(&self, ids: &[SessionId]) {
    let mut bindings: MutexGuard<'_, HashMap<SessionId, SessionScope>> = self.lock();

    for id in ids {
      bindings.remove(id);
    }
  }

  /// The openings bound to `scope`, in no particular order.
  pub fn list(&self, scope: &SessionScope) -> Vec<SessionId> {
    self
      .lock()
      .iter()
      .filter(|(_, bound)| *bound == scope)
      .map(|(id, _)| *id)
      .collect()
  }

  /// Unbinds the openings bound to a scope `is_released` names, answering them.
  pub fn take_where(&self, is_released: impl Fn(&SessionScope) -> bool) -> Vec<SessionId> {
    let mut ids: Vec<SessionId> = Vec::new();

    self.lock().retain(|id, bound| {
      let is_kept: bool = !is_released(bound);

      if !is_kept {
        ids.push(*id);
      }

      is_kept
    });

    ids
  }

  /// Drops what closed openings held on the pool, or here without one.
  pub fn dispose(&self, held: impl Send + 'static) {
    match &self.pool {
      Some(pool) => pool.spawn(move || drop(held)),
      None => drop(held),
    }
  }

  /// The bindings, whatever a panicking caller left: an entry is only ever added or removed whole.
  fn lock(&self) -> MutexGuard<'_, HashMap<SessionId, SessionScope>> {
    self.bindings.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
