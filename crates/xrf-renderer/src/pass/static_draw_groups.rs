/// The static scene's buffers as its draws read them.
pub struct StaticDrawGroups {
  /// Each layout's clusters, by `StaticLayout::get_index`.
  pub layouts: [wgpu::BindGroup; 2],
  pub impostors: wgpu::BindGroup,
}
