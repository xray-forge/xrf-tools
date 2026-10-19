use crate::host::render_bundle::RenderBundle;
use crate::scene::texture::decoded_texture::DecodedTexture;

/// The enhanced rain's maps from the renderer's bundle (`rain/`), uploaded once: the drops a streak carries and the
/// splash, each the screen offset it bends the view by and its cover.
pub struct EnhancedRainMaps {
  pub drop: wgpu::TextureView,
  pub splash: wgpu::TextureView,
}

impl EnhancedRainMaps {
  const DROP: &str = "rain/drop.dds";
  const SPLASH: &str = "rain/splash.dds";

  /// Reads both maps; one the bundle cannot give is `nothing`, which covers nothing, so no drop or splash shows.
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
        .inspect_err(|error| log::error!("The enhanced rain reads no map '{path}': {error}"))
        .ok()
        .filter(|texture| {
          texture.is_flat() || {
            log::error!("The enhanced rain reads no map '{path}': it is not one flat image");
            false
          }
        })
        .map_or_else(|| nothing.clone(), |texture| texture.upload(device, queue, "rain map"))
    };

    Self {
      drop: load(Self::DROP),
      splash: load(Self::SPLASH),
    }
  }
}
