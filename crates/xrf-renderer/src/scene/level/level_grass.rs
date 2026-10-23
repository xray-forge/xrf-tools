use std::sync::Arc;

use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_details::RenderLevelDetails;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::texture::texture_cache::TextureCache;

/// A level's grass (`CDetailManager`): its slots and models as the world last posted them; each view plants and draws
/// it around its own camera (`GrassView`).
#[derive(Default)]
pub struct LevelGrass {
  level: Option<GrassLevel>,
}

impl LevelGrass {
  /// What was posted, none before it is or where the level has no grass.
  pub fn get_level(&self) -> Option<&GrassLevel> {
    self.level.as_ref()
  }

  /// The texture slots its models sample, none before it is posted.
  pub fn list_texture_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self.level.iter().flat_map(|level| level.texture_slots.iter().copied())
  }

  /// Takes the level's grass as read in place of any taken before, asking for its textures; none leaves the level
  /// without grass.
  pub fn replace(
    &mut self,
    device: &wgpu::Device,
    details: Option<RenderLevelDetails>,
    (textures, source): (&mut TextureCache, &Arc<dyn RenderAssetSource>),
  ) {
    self.level = details.map(|details| GrassLevel::new(device, &details, (textures, source)));

    if let Some(level) = &self.level {
      log::info!(
        "Native viewport grass {} models over {}x{} slots",
        level.model_count,
        level.grid[0],
        level.grid[1]
      );
    }
  }
}
