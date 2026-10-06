/// Where a transient lives for the frame: the pool's `ordinal`th resource of its key, which transients of the same key
/// whose lifetimes do not overlap share.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransientSlot<K> {
  pub key: K,
  pub ordinal: usize,
  /// The first and the last surviving pass using it, by position in the compiled order.
  pub first: usize,
  pub last: usize,
}
