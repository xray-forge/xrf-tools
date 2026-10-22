use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use xrf_job::PoolOnceLock;

use crate::core::session::SessionPart;

/// The parts a snapshot keeps, each built the first time anything asks and kept until the snapshot is let go.
pub(crate) struct SessionParts {
  cells: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

/// One part's place: empty until it is built, then what it keeps.
enum SessionPartCell<V> {
  Plain(OnceLock<V>),
  Pooled(PoolOnceLock<V>),
}

impl SessionParts {
  pub fn new() -> Self {
    Self {
      cells: Mutex::new(HashMap::new()),
    }
  }

  /// The store with `value` kept as the part `K` already, for a part the opening read on its way.
  pub fn with<K: SessionPart>(self, value: K::Value) -> Self {
    // A kept part is never built, so even a pooled one takes a plain cell rather than a thread to fill it on.
    self
      .lock()
      .entry(TypeId::of::<K>())
      .or_insert_with(|| Arc::new(SessionPartCell::Plain(OnceLock::from(value))));

    self
  }

  /// The part `K`, built by `build` the first time anything asks.
  ///
  /// # Panics
  ///
  /// Panics where `build` panics, leaving the part unbuilt.
  pub fn get_or_init<K: SessionPart>(&self, build: impl FnOnce() -> K::Value + Send) -> K::Value {
    match &*self.get_cell::<K>() {
      SessionPartCell::Plain(cell) => cell.get_or_init(build).clone(),
      SessionPartCell::Pooled(cell) => cell.get_or_init(build).clone(),
    }
  }

  /// The part `K`, where it was built; asking builds nothing.
  pub fn get<K: SessionPart>(&self) -> Option<K::Value> {
    match &*self.get_cell::<K>() {
      SessionPartCell::Plain(cell) => cell.get().cloned(),
      SessionPartCell::Pooled(cell) => cell.get().cloned(),
    }
  }

  fn get_cell<K: SessionPart>(&self) -> Arc<SessionPartCell<K::Value>> {
    let cell: Arc<dyn Any + Send + Sync> = Arc::clone(self.lock().entry(TypeId::of::<K>()).or_insert_with(|| {
      Arc::new(if K::IS_POOLED {
        SessionPartCell::Pooled(PoolOnceLock::<K::Value>::new())
      } else {
        SessionPartCell::Plain(OnceLock::<K::Value>::new())
      })
    }));

    cell
      .downcast::<SessionPartCell<K::Value>>()
      .unwrap_or_else(|_| unreachable!("a part's cell is filed under its own key"))
  }

  /// The cells, whatever a panicking ask left: a cell is only ever added whole.
  fn lock(&self) -> MutexGuard<'_, HashMap<TypeId, Arc<dyn Any + Send + Sync>>> {
    self.cells.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl Default for SessionParts {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Debug for SessionParts {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("SessionParts")
      .field("parts", &self.lock().len())
      .finish()
  }
}
