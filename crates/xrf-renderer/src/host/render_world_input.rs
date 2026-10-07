use std::time::Instant;

use crate::contract::render_ambient_report::RenderAmbientReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::host::render_sector_failure::RenderSectorFailure;
use crate::host::render_sky_textures::RenderSkyTextures;

/// What the renderer tells the world as it asks for a viewport's frame.
pub struct RenderWorldInput<'a> {
  pub now: Instant,
  /// Seconds since the last frame.
  pub delta: f32,
  /// The viewport's height in CSS pixels, which an orbit's drag is measured against.
  pub height: f32,
  pub options: &'a RenderViewOptions,
  /// The weather's skies, asked for ahead of being drawn.
  pub skies: &'a mut dyn RenderSkyTextures,
  /// Where the level's ambient effects stand, which the weather's report carries.
  pub ambient: Option<RenderAmbientReport>,
  /// The sectors the last frame's updates could not take in.
  pub failures: Vec<RenderSectorFailure>,
}
