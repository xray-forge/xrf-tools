use std::collections::HashMap;

use crate::level::loader_slot::LoaderSlot;

/// Things read once each on the workers by name, the first time anything asks for one, and kept.
pub struct LoaderSlots<T> {
  slots: HashMap<String, LoaderSlot<T>>,
}

impl<T> Default for LoaderSlots<T> {
  fn default() -> Self {
    Self { slots: HashMap::new() }
  }
}

impl<T: Send + 'static> LoaderSlots<T> {
  /// The one named once it is read, starting its read through `spawn` the first time it is asked for; none while it
  /// is on its way or where it is not to be had.
  pub fn get(&mut self, name: &str, spawn: impl FnOnce() -> LoaderSlot<T>) -> Option<&T> {
    if !self.slots.contains_key(name) {
      self.slots.insert(name.to_owned(), spawn());
    }

    self.slots.get_mut(name).and_then(LoaderSlot::poll)
  }

  /// Whether the one named is asked for and its loader has not answered yet.
  #[cfg(test)]
  pub fn is_pending(&self, name: &str) -> bool {
    self.slots.get(name).is_some_and(LoaderSlot::is_pending)
  }
}
