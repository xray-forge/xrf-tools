/// What the water binds of its frame at its fourth group, one for each of its two passes.
pub struct WaterGroups {
  /// The lighting, the water's uniform, the G-buffer's depth, both skies and the nearest water's depth.
  pub surface: wgpu::BindGroup,
  /// The water's uniform alone, which lifts the waves, for the pass writing the nearest water's depth.
  pub depth: wgpu::BindGroup,
}
