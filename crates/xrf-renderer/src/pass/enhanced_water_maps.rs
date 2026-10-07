use crate::host::render_bundle::RenderBundle;
use crate::scene::texture::decoded_texture::DecodedTexture;

pub struct EnhancedWaterMaps {
  pub blue_noise: wgpu::TextureView,
  pub perlin: wgpu::TextureView,
  pub normal: wgpu::TextureView,
  pub wind: wgpu::TextureView,
  pub caustics: wgpu::TextureView,
  pub height: wgpu::TextureView,
  pub ripples: wgpu::TextureView,
}

impl EnhancedWaterMaps {
  const BLUE_NOISE: &str = "water/blue_noise.dds";
  const PERLIN: &str = "water/perlin.dds";
  const NORMAL: &str = "water/normal.dds";
  const WIND: &str = "water/wind.dds";
  const CAUSTICS: &str = "water/caustics.dds";
  const HEIGHT: &str = "water/height.dds";
  const RIPPLES: &str = "water/ripples.dds";

  /// Reads every map; one the bundle cannot give is `nothing`: the reflection's march unjittered, its blur unmixed,
  /// the waves flat.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bundle: &dyn RenderBundle,
    nothing: &wgpu::TextureView,
  ) -> Self {
    let load = |path: &str| -> wgpu::TextureView {
      bundle
        .read_bundled(path)
        .and_then(|bytes| DecodedTexture::from_dds(&bytes))
        .inspect_err(|error| log::error!("The enhanced water reads no map '{path}': {error}"))
        .ok()
        .map_or_else(|| nothing.clone(), |texture| texture.upload(device, queue))
    };

    Self {
      blue_noise: load(Self::BLUE_NOISE),
      perlin: load(Self::PERLIN),
      normal: load(Self::NORMAL),
      wind: load(Self::WIND),
      caustics: load(Self::CAUSTICS),
      height: load(Self::HEIGHT),
      ripples: load(Self::RIPPLES),
    }
  }
}
