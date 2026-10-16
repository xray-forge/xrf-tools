use std::time::{Duration, Instant};

use crate::contract::render_view_options::RenderViewOptions;
use crate::host::render_sky_textures::RenderSkyTextures;

/// What the renderer tells the world as it asks for a viewport's frame.
pub struct RenderViewInput<'a> {
  pub now: Instant,
  /// Time since the last frame.
  pub delta: Duration,
  /// The viewport's height in CSS pixels, which an orbit's drag is measured against.
  pub height: f32,
  pub options: &'a RenderViewOptions,
  /// The weather's skies, asked for ahead of being drawn.
  pub skies: &'a mut dyn RenderSkyTextures,
}
