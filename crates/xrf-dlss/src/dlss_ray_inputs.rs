use glam::Mat4;

/// What a view's frame gives DLSS ray reconstruction to denoise and upscale: what super resolution takes, and the
/// guides telling the network what each pixel's surface is.
pub struct DlssRayInputs<'a> {
  /// The frame as drawn, of linear light, at the render size.
  pub color: &'a wgpu::TextureView,
  /// Its depth, reversed: one at the near plane.
  pub depth: &'a wgpu::TextureView,
  /// Each pixel's motion at the render size, which `motion_scale` turns into render pixels from where it stands now
  /// back to where it stood the frame before.
  pub motion: &'a wgpu::TextureView,
  pub motion_scale: [f32; 2],
  /// The share of light each surface scatters diffusely.
  pub diffuse_albedo: &'a wgpu::TextureView,
  /// The share of the light around it each surface reflects: its environment BRDF over its reflectance.
  pub specular_albedo: &'a wgpu::TextureView,
  /// Each surface's world normal, its linear roughness in the fourth channel.
  pub normals: &'a wgpu::TextureView,
  /// How far each surface's reflection ray went before it met what it reflects, in metres.
  pub specular_hit_distance: &'a wgpu::TextureView,
  /// The unjittered view and projection the hit distances are placed through.
  pub world_to_view: Mat4,
  pub view_to_clip: Mat4,
  /// Where this frame's samples sit within their pixels, in render pixels.
  pub jitter: [f32; 2],
  /// The render size, within the reconstructor's range.
  pub render: (u32, u32),
  /// The texture DLSS writes the reconstructed frame into, at the output's size, bound as storage.
  pub output: &'a wgpu::TextureView,
  /// Whether the frames before are forgotten: the first frame, or one after a cut.
  pub is_reset: bool,
}

impl DlssRayInputs<'_> {
  /// The views NGX reads.
  pub(crate) fn get_read(&self) -> [&wgpu::TextureView; 7] {
    [
      self.color,
      self.depth,
      self.motion,
      self.diffuse_albedo,
      self.specular_albedo,
      self.normals,
      self.specular_hit_distance,
    ]
  }
}
