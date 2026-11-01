use crate::lighting::render_sky::RenderSky;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The textures a weather's skies are drawn with, as the world asks for them: none is drawn before it is resident.
pub trait RenderSkyTextures {
  /// Asks for every texture a sky samples, answering whether all are resident.
  fn request_sky(&mut self, sky: &RenderSky) -> bool;

  /// Asks for a texture ahead of the frame that draws it.
  fn prefetch(&mut self, reference: &str, kind: WeatherTextureKind);
}
