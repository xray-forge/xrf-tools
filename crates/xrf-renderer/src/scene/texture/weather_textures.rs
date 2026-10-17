use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The weather textures as one source reads them: a scene's, whose scope tells its files apart from another
/// installation's of the same reference.
#[derive(Clone, Copy)]
pub struct WeatherTextures<'a> {
  cache: &'a WeatherTextureCache,
  scope: &'a str,
}

impl<'a> WeatherTextures<'a> {
  pub fn new(cache: &'a WeatherTextureCache, scope: &'a str) -> Self {
    Self { cache, scope }
  }

  /// Whether a texture is up or given up on for good; none counts as settled.
  pub fn is_settled(&self, reference: Option<&str>) -> bool {
    self.cache.is_settled(self.scope, reference)
  }

  /// Whether a texture is up, rather than standing in for itself.
  pub fn is_up(&self, reference: &str) -> bool {
    self.cache.is_up(self.scope, reference)
  }

  /// What samples a texture: its own once up, its kind's placeholder before, or for none.
  pub fn get_view(&self, reference: Option<&str>, kind: WeatherTextureKind) -> &'a wgpu::TextureView {
    self.cache.get_view(self.scope, reference, kind)
  }
}
