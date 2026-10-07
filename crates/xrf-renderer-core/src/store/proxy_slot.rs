/// Where a store's slot points: the item's place among the dense records, or `FREE`, and the generation a handle to it
/// must carry.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ProxySlot {
  pub dense: u32,
  pub generation: u32,
}

impl ProxySlot {
  /// A slot no item holds.
  pub const FREE: u32 = u32::MAX;
}
