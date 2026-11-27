use crate::id::id_entry::IdEntry;
use crate::id::object_id::ObjectId;

/// A table of values by object id: a dense vector over the slots, so a lookup is an index and iteration runs in id order,
/// the same every run, which a step that must repeat relies on (no hash map iterates in a step). Each value keeps its
/// id, so an id of a slot's earlier object finds nothing.
///
/// Rows are added and taken only by the registry, which keeps every table to the objects that exist; outside it a table
/// is read, or its values changed in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdMap<T> {
  slots: Vec<Option<IdEntry<T>>>,
  count: usize,
}

impl<T> Default for IdMap<T> {
  fn default() -> Self {
    Self {
      slots: Vec::new(),
      count: 0,
    }
  }
}

impl<T> IdMap<T> {
  /// Puts `value` at `id`, answering what stood there for the same id. A row an earlier object of the slot left is
  /// replaced, never answered.
  pub(crate) fn insert(&mut self, id: ObjectId, value: T) -> Option<T> {
    let index: usize = id.get_slot();

    if self.slots.len() <= index {
      self.slots.resize_with(index + 1, || None);
    }

    match self.slots[index].replace(IdEntry { id, value }) {
      Some(previous) if previous.id == id => Some(previous.value),
      Some(_) => None,
      None => {
        self.count += 1;

        None
      }
    }
  }

  /// Takes the value at `id` out, answering it.
  pub(crate) fn remove(&mut self, id: ObjectId) -> Option<T> {
    let slot: &mut Option<IdEntry<T>> = self.slots.get_mut(id.get_slot())?;

    if slot.as_ref().is_none_or(|entry| entry.id != id) {
      return None;
    }

    self.count -= 1;

    slot.take().map(|entry| entry.value)
  }

  pub fn get(&self, id: ObjectId) -> Option<&T> {
    self
      .slots
      .get(id.get_slot())?
      .as_ref()
      .filter(|entry| entry.id == id)
      .map(|entry| &entry.value)
  }

  pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut T> {
    self
      .slots
      .get_mut(id.get_slot())?
      .as_mut()
      .filter(|entry| entry.id == id)
      .map(|entry| &mut entry.value)
  }

  pub fn contains(&self, id: ObjectId) -> bool {
    self.get(id).is_some()
  }

  /// The id whose value stands in slot `index`, whatever its generation.
  pub(crate) fn find_in_slot(&self, index: u32) -> Option<ObjectId> {
    let slot: usize = usize::try_from(index).ok()?;

    self.slots.get(slot)?.as_ref().map(|entry| entry.id)
  }

  /// How many values it holds.
  pub fn len(&self) -> usize {
    self.count
  }

  pub fn is_empty(&self) -> bool {
    self.count == 0
  }

  /// Every value with its id, in id order.
  pub fn iter(&self) -> impl Iterator<Item = (ObjectId, &T)> {
    self.iter_from(ObjectId::of_index(0))
  }

  /// Every value from `id`'s slot on with its id, in id order, that slot's own first if it holds one: where a walk that
  /// stopped takes up again.
  pub fn iter_from(&self, id: ObjectId) -> impl Iterator<Item = (ObjectId, &T)> {
    let start: usize = id.get_slot().min(self.slots.len());

    self.slots[start..]
      .iter()
      .filter_map(|slot| slot.as_ref().map(|entry| (entry.id, &entry.value)))
  }

  /// Every value with its id, in id order, to change.
  pub fn iter_mut(&mut self) -> impl Iterator<Item = (ObjectId, &mut T)> {
    self
      .slots
      .iter_mut()
      .filter_map(|slot| slot.as_mut().map(|entry| (entry.id, &mut entry.value)))
  }
}
