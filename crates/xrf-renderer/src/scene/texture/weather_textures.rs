use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The weather textures as one source reads them: a scene's, whose scope tells its files apart from another
/// installation's of the same reference.
#[derive(Clone, Copy)]
pub(crate) struct WeatherTextures<'a> {
  cache: &'a WeatherTextureCache,
  scope: &'a str,
}

impl<'a> WeatherTextures<'a> {
  pub(crate) fn new(cache: &'a WeatherTextureCache, scope: &'a str) -> Self {
    Self { cache, scope }
  }

  /// Whether a texture is up or given up on for good; none counts as settled.
  pub(crate) fn is_settled(&self, reference: Option<&str>) -> bool {
    self.cache.is_settled(self.scope, reference)
  }

  /// Whether a texture is up, rather than standing in for itself.
  pub(crate) fn is_up(&self, reference: &str) -> bool {
    self.cache.is_up(self.scope, reference)
  }

  /// A texture's size, a face's for a cube, once up.
  pub(crate) fn get_size(&self, reference: &str) -> Option<(u32, u32)> {
    self.cache.get_size(self.scope, reference)
  }

  /// What samples a texture: its own once up, its kind's placeholder before, or for none.
  pub(crate) fn get_view(&self, reference: Option<&str>, kind: WeatherTextureKind) -> &'a wgpu::TextureView {
    self.cache.get_view(self.scope, reference, kind)
  }
}
