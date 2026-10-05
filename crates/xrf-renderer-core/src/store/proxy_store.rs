use crate::store::proxy_handle::ProxyHandle;
use crate::store::proxy_slot::ProxySlot;

/// One kind of a scene's items, kept as dense records the GPU mirrors element for element: an add appends, a remove
/// moves the last record into the gap, and every record added, changed or moved is noted for the mirror to upload.
/// Items are reached by generational handles, which outlive neither their item nor a slot reused since.
pub struct ProxyStore<T> {
  items: Vec<T>,
  /// The slot each dense record is held through.
  owners: Vec<u32>,
  slots: Vec<ProxySlot>,
  /// Slots no item holds, reused before new ones.
  free: Vec<u32>,
  /// Dense records added, changed or moved since the mirror last took them.
  changed: Vec<u32>,
}

impl<T> Default for ProxyStore<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> ProxyStore<T> {
  pub fn new() -> Self {
    Self {
      items: Vec::new(),
      owners: Vec::new(),
      slots: Vec::new(),
      free: Vec::new(),
      changed: Vec::new(),
    }
  }

  pub fn len(&self) -> usize {
    self.items.len()
  }

  pub fn is_empty(&self) -> bool {
    self.items.is_empty()
  }

  /// Puts an item in, at the end of the dense records.
  pub fn add(&mut self, item: T) -> ProxyHandle<T> {
    let dense: u32 = self.items.len() as u32;
    let slot: u32 = match self.free.pop() {
      Some(slot) => {
        self.slots[slot as usize].dense = dense;
        slot
      }
      None => {
        self.slots.push(ProxySlot { dense, generation: 0 });
        (self.slots.len() - 1) as u32
      }
    };

    self.items.push(item);
    self.owners.push(slot);
    self.changed.push(dense);

    ProxyHandle::new(slot, self.slots[slot as usize].generation)
  }

  /// Takes an item out, the last record moving into its place; nothing for a handle whose item is gone.
  pub fn remove(&mut self, handle: ProxyHandle<T>) -> Option<T> {
    let dense: u32 = self.get_index(handle)?;
    let last: u32 = (self.items.len() - 1) as u32;
    let item: T = self.items.swap_remove(dense as usize);

    self.owners.swap_remove(dense as usize);

    if dense != last {
      self.slots[self.owners[dense as usize] as usize].dense = dense;
      self.changed.push(dense);
    }

    let slot: &mut ProxySlot = &mut self.slots[handle.slot as usize];

    slot.dense = ProxySlot::FREE;
    slot.generation = slot.generation.wrapping_add(1);
    self.free.push(handle.slot);

    Some(item)
  }

  pub fn contains(&self, handle: ProxyHandle<T>) -> bool {
    self.get_index(handle).is_some()
  }

  /// Where an item's record stands among the dense records, which is where the GPU mirror holds it.
  pub fn get_index(&self, handle: ProxyHandle<T>) -> Option<u32> {
    self
      .slots
      .get(handle.slot as usize)
      .filter(|slot| slot.generation == handle.generation && slot.dense != ProxySlot::FREE)
      .map(|slot| slot.dense)
  }

  pub fn get(&self, handle: ProxyHandle<T>) -> Option<&T> {
    self.get_index(handle).map(|dense| &self.items[dense as usize])
  }

  /// The item to change, its record noted for the mirror.
  pub fn get_mut(&mut self, handle: ProxyHandle<T>) -> Option<&mut T> {
    let dense: u32 = self.get_index(handle)?;

    self.changed.push(dense);

    Some(&mut self.items[dense as usize])
  }

  /// The dense records, in the order the GPU mirror holds them.
  pub fn as_slice(&self) -> &[T] {
    &self.items
  }

  /// Each item with the handle it is held by, in dense order.
  pub fn iter(&self) -> impl Iterator<Item = (ProxyHandle<T>, &T)> {
    self.items.iter().zip(&self.owners).map(|(item, slot)| {
      (
        ProxyHandle::new(*slot, self.slots[*slot as usize].generation),
        item,
      )
    })
  }

  /// The dense records added, changed or moved since the last call, each once and in order, still standing.
  pub fn take_changed(&mut self) -> Vec<u32> {
    let mut changed: Vec<u32> = std::mem::take(&mut self.changed);
    let count: u32 = self.items.len() as u32;

    changed.retain(|dense| *dense < count);
    changed.sort_unstable();
    changed.dedup();
    changed
  }

  /// Notes every record as changed, for a mirror that lost what it held (a device made again).
  pub fn mark_all_changed(&mut self) {
    self.changed = (0..self.items.len() as u32).collect();
  }
}
