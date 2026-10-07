use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// What a scene's caller holds an item of a store by: its slot and the generation the slot was at when the item went
/// in, so a handle kept past its item's removal answers nothing rather than whatever took the slot since.
pub struct ProxyHandle<T> {
  pub(crate) slot: u32,
  pub(crate) generation: u32,
  item: PhantomData<fn() -> T>,
}

impl<T> ProxyHandle<T> {
  pub(crate) fn new(slot: u32, generation: u32) -> Self {
    Self {
      slot,
      generation,
      item: PhantomData,
    }
  }
}

impl<T> Clone for ProxyHandle<T> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T> Copy for ProxyHandle<T> {}

impl<T> PartialEq for ProxyHandle<T> {
  fn eq(&self, other: &Self) -> bool {
    self.slot == other.slot && self.generation == other.generation
  }
}

impl<T> Eq for ProxyHandle<T> {}

impl<T> Hash for ProxyHandle<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.slot.hash(state);
    self.generation.hash(state);
  }
}

impl<T> fmt::Debug for ProxyHandle<T> {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(formatter, "ProxyHandle({}v{})", self.slot, self.generation)
  }
}
