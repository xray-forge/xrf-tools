use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use xrf_job::PoolOnceLock;
use xrf_vfs::{XrayInputRecord, XraySearch, XrayVfs};

use crate::core::session::SessionPart;

/// The parts a snapshot keeps, each built the first time anything asks and kept until the snapshot is let go, with the
/// inputs its build read.
pub(crate) struct SessionParts {
  slots: Mutex<HashMap<TypeId, SessionPartSlot>>,
}

/// One part's place: its cell, and what was read for it.
#[derive(Clone)]
struct SessionPartSlot {
  cell: Arc<dyn Any + Send + Sync>,
  inputs: XrayInputRecord,
  /// The part's [`SessionPart::NAME`], which a report and a change event name it by.
  name: &'static str,
  /// Whether the cell holds a value, asked of the erased cell by the one function that knows its type.
  is_built: fn(&(dyn Any + Send + Sync)) -> bool,
}

/// One part's value: empty until it is built, then what it keeps.
enum SessionPartCell<V> {
  Plain(OnceLock<V>),
  Pooled(PoolOnceLock<V>),
}

impl SessionParts {
  pub fn new() -> Self {
    Self {
      slots: Mutex::new(HashMap::new()),
    }
  }

  /// The store with `value` kept as the part `K` already, read from `inputs`, for a part the opening read on its way.
  pub fn with<K: SessionPart>(self, value: K::Value, inputs: XrayInputRecord) -> Self {
    // A kept part is never built, so even a pooled one takes a plain cell rather than a thread to fill it on.
    self.lock().entry(TypeId::of::<K>()).or_insert_with(|| SessionPartSlot {
      cell: Arc::new(SessionPartCell::Plain(OnceLock::from(value))),
      inputs,
      name: K::NAME,
      is_built: SessionPartCell::<K::Value>::is_built_erased,
    });

    self
  }

  /// The store a next snapshot starts with over `vfs`: every built part whose inputs are all current there, shared
  /// rather than copied, and the names of the built parts that are not, sorted. A part still building is carried by
  /// neither, since what it reads is not known yet; the next snapshot builds it afresh when asked.
  pub fn retain_current(&self, vfs: &XrayVfs) -> (SessionParts, Vec<&'static str>) {
    let mut carried: HashMap<TypeId, SessionPartSlot> = HashMap::new();
    let mut stale: Vec<&'static str> = Vec::new();

    for (key, slot) in self.lock().iter() {
      if !(slot.is_built)(slot.cell.as_ref()) {
        continue;
      }

      if slot.inputs.is_current_in(vfs) {
        carried.insert(*key, slot.clone());
      } else {
        stale.push(slot.name);
      }
    }

    stale.sort_unstable();

    (
      SessionParts {
        slots: Mutex::new(carried),
      },
      stale,
    )
  }

  /// The names of every part built, sorted: what goes stale at once when the value they derive from is derived again.
  pub fn list_built(&self) -> Vec<&'static str> {
    let mut built: Vec<&'static str> = self
      .lock()
      .values()
      .filter(|slot| (slot.is_built)(slot.cell.as_ref()))
      .map(|slot| slot.name)
      .collect();

    built.sort_unstable();
    built
  }

  /// The part `K`, built by `build` the first time anything asks, for a part that reads nothing of the project.
  ///
  /// # Panics
  ///
  /// Panics where `build` panics, leaving the part unbuilt.
  pub fn get_or_init<K: SessionPart>(&self, build: impl FnOnce() -> K::Value + Send) -> K::Value {
    self.get_slot::<K>().0.get_or_init(build)
  }

  /// The part `K`, built by `build` the first time anything asks through a search recording what it reads into the
  /// part's inputs.
  ///
  /// # Panics
  ///
  /// Panics where `build` panics, leaving the part unbuilt.
  pub fn get_or_record<K: SessionPart>(
    &self,
    search: &XraySearch,
    build: impl FnOnce(&XraySearch) -> K::Value + Send,
  ) -> K::Value {
    let (cell, inputs) = self.get_slot::<K>();
    let value: K::Value = cell.get_or_init(|| build(&search.clone().with_recording(&inputs)));

    search.record_inputs(&inputs);

    value
  }

  /// Runs `read` through a search recording into the inputs of the part `K`, for a part whose values are read after it
  /// is built, one key at a time.
  ///
  /// As [`Self::get_or_record`], the part's inputs join the asker's where `search` records.
  pub fn record<K: SessionPart, T>(&self, search: &XraySearch, read: impl FnOnce(&XraySearch) -> T) -> T {
    let inputs: XrayInputRecord = self.get_slot::<K>().1;
    let read: T = read(&search.clone().with_recording(&inputs));

    search.record_inputs(&inputs);

    read
  }

  /// The part `K`, where it was built; asking builds nothing.
  pub fn get<K: SessionPart>(&self) -> Option<K::Value> {
    self.find_slot::<K>()?.0.get()
  }

  /// The inputs the part `K` was built from, where anything asked for it.
  #[cfg(test)]
  pub fn get_inputs<K: SessionPart>(&self) -> Option<XrayInputRecord> {
    self.find_slot::<K>().map(|(_, inputs)| inputs)
  }

  /// Every part asked for, by its key's type name, with the inputs it was built from.
  #[cfg(test)]
  pub fn list_inputs(&self) -> Vec<(&'static str, XrayInputRecord)> {
    self
      .lock()
      .values()
      .map(|slot| (slot.name, slot.inputs.clone()))
      .collect()
  }

  fn get_slot<K: SessionPart>(&self) -> (Arc<SessionPartCell<K::Value>>, XrayInputRecord) {
    let slot: SessionPartSlot = self
      .lock()
      .entry(TypeId::of::<K>())
      .or_insert_with(|| SessionPartSlot {
        cell: Arc::new(if K::IS_POOLED {
          SessionPartCell::Pooled(PoolOnceLock::<K::Value>::new())
        } else {
          SessionPartCell::Plain(OnceLock::<K::Value>::new())
        }),
        inputs: XrayInputRecord::new(),
        name: K::NAME,
        is_built: SessionPartCell::<K::Value>::is_built_erased,
      })
      .clone();

    Self::downcast::<K>(slot)
  }

  fn find_slot<K: SessionPart>(&self) -> Option<(Arc<SessionPartCell<K::Value>>, XrayInputRecord)> {
    let slot: SessionPartSlot = self.lock().get(&TypeId::of::<K>())?.clone();

    Some(Self::downcast::<K>(slot))
  }

  fn downcast<K: SessionPart>(slot: SessionPartSlot) -> (Arc<SessionPartCell<K::Value>>, XrayInputRecord) {
    let cell: Arc<SessionPartCell<K::Value>> = slot
      .cell
      .downcast::<SessionPartCell<K::Value>>()
      .unwrap_or_else(|_| unreachable!("a part's cell is filed under its own key"));

    (cell, slot.inputs)
  }

  /// The slots, whatever a panicking ask left: a slot is only ever added whole.
  fn lock(&self) -> MutexGuard<'_, HashMap<TypeId, SessionPartSlot>> {
    self.slots.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl<V: Clone + Send> SessionPartCell<V> {
  fn get_or_init(&self, build: impl FnOnce() -> V + Send) -> V {
    match self {
      SessionPartCell::Plain(cell) => cell.get_or_init(build).clone(),
      SessionPartCell::Pooled(cell) => cell.get_or_init(build).clone(),
    }
  }

  fn get(&self) -> Option<V> {
    match self {
      SessionPartCell::Plain(cell) => cell.get().cloned(),
      SessionPartCell::Pooled(cell) => cell.get().cloned(),
    }
  }
}

impl<V: Send + Sync + 'static> SessionPartCell<V> {
  /// Whether `cell`, a slot's erased cell of this type, holds a value.
  fn is_built_erased(cell: &(dyn Any + Send + Sync)) -> bool {
    match cell.downcast_ref::<Self>() {
      Some(SessionPartCell::Plain(cell)) => cell.get().is_some(),
      Some(SessionPartCell::Pooled(cell)) => cell.get().is_some(),
      None => unreachable!("a part's cell is filed under its own key"),
    }
  }
}

impl Default for SessionParts {
  fn default() -> Self {
    Self::new()
  }
}

impl fmt::Debug for SessionParts {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let names: Vec<&'static str> = self.lock().values().map(|slot| slot.name).collect();

    formatter.debug_struct("SessionParts").field("parts", &names).finish()
  }
}
