use xrf_material::XraySurfaceDraw;

/// A frame's strike as the view draws it: the textures of its model, its top glow and its centre glow, in that order,
/// and how each draws; the bolt model it draws, by its index among the weather's, and that model's indices.
pub struct ThunderStrike {
  pub textures: [wgpu::TextureView; 3],
  pub draws: [XraySurfaceDraw; 3],
  pub model: Option<usize>,
  pub model_indices: u32,
}
