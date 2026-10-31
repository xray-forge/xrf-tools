/// One way the engine's bloom is blurred, `bloom_filter.ps`.
#[derive(Clone, Copy, Debug)]
pub struct BloomFilter {
  /// Whether it blurs across the target; down otherwise.
  pub is_across: bool,
  /// The blur's radius across, `kernel_g` texels.
  pub radius: f32,
  /// What the weights sum to, `kernel_scale`.
  pub strength: f32,
  /// The frame's height over its width, which the radius down is taken by.
  pub aspect: f32,
  /// Whether only the side behind is read, as Monolith's filter does.
  pub is_one_sided: bool,
}
