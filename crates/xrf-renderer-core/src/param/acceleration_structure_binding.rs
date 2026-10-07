use std::sync::atomic::{AtomicU64, Ordering};

/// A top-level acceleration structure a pass's rays are traced against, and the key that names it to the bind group
/// cache: a structure is not a resource the cache can hold, so each one made takes a key of its own.
#[derive(Clone, Copy, Debug)]
pub struct AccelerationStructureBinding<'a> {
  pub tlas: &'a wgpu::Tlas,
  pub key: u64,
}

impl<'a> AccelerationStructureBinding<'a> {
  pub fn new(tlas: &'a wgpu::Tlas, key: u64) -> Self {
    Self { tlas, key }
  }

  /// A key no structure made before holds, for one just made.
  pub fn next_key() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);

    NEXT.fetch_add(1, Ordering::Relaxed)
  }
}
