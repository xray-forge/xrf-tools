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
  /// How far each pixel is reconstructed from this frame rather than the history, from nothing to one, where its look
  /// changes in place from frame to frame (a puddle's ripples): none to leave every pixel to the network.
  pub bias_current_color: Option<&'a wgpu::TextureView>,
  /// The frame before and after its transparent layers drew (water, glass, particles, rain), at the render size, so
  /// the network tells them from what it reconstructs: none to leave them in the frame.
  pub transparency: Option<(&'a wgpu::TextureView, &'a wgpu::TextureView)>,
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
  pub(crate) fn get_read(&self) -> Vec<&wgpu::TextureView> {
    let mut views: Vec<&wgpu::TextureView> = vec![
      self.color,
      self.depth,
      self.motion,
      self.diffuse_albedo,
      self.specular_albedo,
      self.normals,
      self.specular_hit_distance,
    ];

    views.extend(self.bias_current_color);
    views.extend(
      self
        .transparency
        .into_iter()
        .flat_map(|(before, after)| [before, after]),
    );

    views
  }
}
