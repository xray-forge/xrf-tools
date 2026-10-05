use std::time::Instant;

use xrf_renderer_core::ProxyHandle;

use crate::contract::render_view_options::RenderViewOptions;
use crate::host::render_sector_failure::RenderSectorFailure;
use crate::host::render_sky_textures::RenderSkyTextures;
use crate::scene::level::particle_emitter_proxy::ParticleEmitterProxy;
use crate::scene::level::placed_effect::PlacedEffect;

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
  /// The sectors the last frame's updates could not take in.
  pub failures: Vec<RenderSectorFailure>,
  /// The effects whose particles stopped playing last frame.
  pub finished_effects: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
}
