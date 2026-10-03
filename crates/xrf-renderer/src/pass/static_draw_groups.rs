use crate::scene::static_scene::static_layout::StaticLayout;

/// The static scene's buffers as its draws read them.
pub struct StaticDrawGroups {
  /// Each layout's clusters, by `StaticLayout::get_index`.
  pub layouts: [wgpu::BindGroup; StaticLayout::COUNT],
  pub impostors: wgpu::BindGroup,
}
