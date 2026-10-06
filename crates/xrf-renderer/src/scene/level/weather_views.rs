/// The weather textures this frame's rain, wet surfaces and strike draw with, as the cache holds them when the frame is
/// prepared, each its kind's placeholder until its file is up.
#[derive(Default)]
pub struct WeatherViews {
  /// The streak's and the splash's.
  pub rain: Option<[wgpu::TextureView; 2]>,
  /// The wet surfaces' splash volume and flow.
  pub wet: Option<[wgpu::TextureView; 2]>,
  /// The strike's model's, its top glow's and its centre glow's, and the bolt model it draws, by its index among the
  /// weather's.
  pub thunder: Option<([wgpu::TextureView; 3], Option<usize>)>,
}
