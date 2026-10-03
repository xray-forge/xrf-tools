/// What one viewport's lighting passes bind, made again with its targets.
pub struct ViewLightGroups {
  pub sun: wgpu::BindGroup,
  /// The local lights' binning, then their shading.
  pub lights: [wgpu::BindGroup; 2],
  /// The ambient occlusion's two, alternating.
  pub occlusion: [wgpu::BindGroup; 2],
  pub combine: wgpu::BindGroup,
  pub haze: wgpu::BindGroup,
  pub exposure: wgpu::BindGroup,
  pub present: wgpu::BindGroup,
}
