use std::marker::PhantomData;

use crate::store::proxy_handle::ProxyHandle;

/// Hands out a store's handles where its items are made, ahead of the store that holds them: a world allocating the
/// handles of what it posts to a scene, which inserts each item at its handle when it applies the post.
pub struct ProxyAllocator<T> {
  generations: Vec<u32>,
  live: Vec<bool>,
  /// Slots no handle holds, reused before new ones.
  free: Vec<u32>,
  item: PhantomData<fn() -> T>,
}

impl<T> Default for ProxyAllocator<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> ProxyAllocator<T> {
  pub fn new() -> Self {
    Self {
      generations: Vec::new(),
      live: Vec::new(),
      free: Vec::new(),
      item: PhantomData,
    }
  }

  pub fn allocate(&mut self) -> ProxyHandle<T> {
    let slot: u32 = match self.free.pop() {
      Some(slot) => slot,
      None => {
        self.generations.push(0);
        self.live.push(false);
        (self.generations.len() - 1) as u32
      }
    };

    self.live[slot as usize] = true;

    ProxyHandle::new(slot, self.generations[slot as usize])
  }

  /// Gives a handle back, its slot's next handle a generation on; false for one already given back.
  pub fn free(&mut self, handle: ProxyHandle<T>) -> bool {
    if !self.contains(handle) {
      return false;
    }

    let slot: usize = handle.slot as usize;

    self.live[slot] = false;
    self.generations[slot] = self.generations[slot].wrapping_add(1);
    self.free.push(handle.slot);

    true
  }

  pub fn contains(&self, handle: ProxyHandle<T>) -> bool {
    let slot: usize = handle.slot as usize;

    self.live.get(slot).copied().unwrap_or(false) && self.generations[slot] == handle.generation
  }
}
