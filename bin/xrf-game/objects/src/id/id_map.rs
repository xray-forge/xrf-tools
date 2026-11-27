use crate::id::object_id::ObjectId;

/// A table of values by object id: a dense vector over the sixteen-bit id space, so a lookup is an index and iteration
/// runs in id order, the same every run, which a step that must repeat relies on (no hash map iterates in a step).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdMap<T> {
  slots: Vec<Option<T>>,
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
  /// Puts `value` at `id`, answering what stood there.
  pub fn insert(&mut self, id: ObjectId, value: T) -> Option<T> {
    let index: usize = id.get_index();

    if self.slots.len() <= index {
      self.slots.resize_with(index + 1, || None);
    }

    let previous: Option<T> = self.slots[index].replace(value);

    if previous.is_none() {
      self.count += 1;
    }

    previous
  }

  /// Takes the value at `id` out, answering it.
  pub fn remove(&mut self, id: ObjectId) -> Option<T> {
    let previous: Option<T> = self.slots.get_mut(id.get_index()).and_then(Option::take);

    if previous.is_some() {
      self.count -= 1;
    }

    previous
  }

  pub fn get(&self, id: ObjectId) -> Option<&T> {
    self.slots.get(id.get_index()).and_then(Option::as_ref)
  }

  pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut T> {
    self.slots.get_mut(id.get_index()).and_then(Option::as_mut)
  }

  pub fn contains(&self, id: ObjectId) -> bool {
    self.get(id).is_some()
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
    self.slots.iter().enumerate().filter_map(|(index, slot)| {
      let value: &T = slot.as_ref()?;

      Some((ObjectId(u16::try_from(index).ok()?), value))
    })
  }

  /// Every value with its id, in id order, to change.
  pub fn iter_mut(&mut self) -> impl Iterator<Item = (ObjectId, &mut T)> {
    self.slots.iter_mut().enumerate().filter_map(|(index, slot)| {
      let value: &mut T = slot.as_mut()?;

      Some((ObjectId(u16::try_from(index).ok()?), value))
    })
  }
}
