//! The arithmetic of WGSL's layout rules, in constants, for what `#[derive(ShaderStruct)]` generates.

/// `offset` rounded up to the next multiple of `align`.
pub const fn round_up(align: u64, offset: u64) -> u64 {
  offset.div_ceil(align) * align
}

/// The largest of `values`.
pub const fn max_of(values: &[u64]) -> u64 {
  let mut largest: u64 = 0;
  let mut index: usize = 0;

  while index < values.len() {
    if values[index] > largest {
      largest = values[index];
    }

    index += 1;
  }

  largest
}
