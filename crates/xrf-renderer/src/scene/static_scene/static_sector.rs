use std::ops::Range;

/// One sector as the scene holds it: the slots it put, which a pick names it by, and what it weighed.
#[derive(Clone, Debug)]
pub struct StaticSector {
  pub sector: u32,
  pub slots: Range<u32>,
  /// Bytes of its pack, which is what a residency budget spends.
  pub bytes: u64,
}
