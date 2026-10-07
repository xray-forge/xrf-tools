use xrf_renderer_core::ProxyHandle;

use crate::host::render_sector_failure::RenderSectorFailure;
use crate::scene::level::particle_emitter_proxy::ParticleEmitterProxy;
use crate::scene::level::placed_effect::PlacedEffect;

/// What a scene tells the world of the last frame: the sectors it could not take in, and the effects whose particles
/// stopped playing.
#[derive(Default)]
pub struct RenderSceneFeedback {
  pub failures: Vec<RenderSectorFailure>,
  pub finished_effects: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
}
