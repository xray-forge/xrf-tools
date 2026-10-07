use std::sync::Arc;

use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_sky_textures::RenderSkyTextures;
use crate::lighting::render_sky::RenderSky;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The weather textures a viewport's world asks for, read from its level's source; with no GPU or no level every sky
/// counts as resident, so the weather plays on.
pub struct SkyTextureRequests<'a> {
  pub cache: Option<&'a mut WeatherTextureCache>,
  pub source: Option<Arc<dyn RenderAssetSource>>,
  /// Whether the clouds are drawn, so the sky waits for their layers too.
  pub is_clouded: bool,
}

impl RenderSkyTextures for SkyTextureRequests<'_> {
  fn request_sky(&mut self, sky: &RenderSky) -> bool {
    match (self.cache.as_deref_mut(), &self.source) {
      (Some(cache), Some(source)) => cache.request_sky(sky, self.is_clouded, source),
      _ => true,
    }
  }

  fn prefetch(&mut self, reference: &str, kind: WeatherTextureKind) {
    if let (Some(cache), Some(source)) = (self.cache.as_deref_mut(), &self.source) {
      cache.request(reference, kind, source);
    }
  }
}
