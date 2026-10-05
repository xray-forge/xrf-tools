/// What the water binds of its frame, for each of its passes.
pub struct WaterGroups {
  /// The water's uniform alone, which lifts the waves, for the pass writing the nearest water's depth.
  pub depth: wgpu::BindGroup,
  /// The surface's, one for each history the reflection writes into: the lighting, the uniform, the G-buffer's depth,
  /// both skies, the nearest water's depth, the scene before it, and the reflection blurred and clear.
  pub surface: [wgpu::BindGroup; 2],
  /// While the enhanced water reflects: the reflection pass's for each history it writes, reading the other.
  pub reflection: Option<[wgpu::BindGroup; 2]>,
  /// While the enhanced water reflects: its blur across from each history, then down.
  pub blur: Option<([wgpu::BindGroup; 2], wgpu::BindGroup)>,
}
