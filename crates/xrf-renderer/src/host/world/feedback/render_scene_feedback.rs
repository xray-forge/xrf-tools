use xrf_renderer_core::ProxyHandle;

use crate::host::level::particles::placed_effect::PlacedEffect;
use crate::host::world::feedback::render_model_failure::RenderModelFailure;
use crate::host::world::feedback::render_sector_failure::RenderSectorFailure;
use crate::scene::level::particles::placed::particle_emitter_proxy::ParticleEmitterProxy;

/// What a scene tells the world of the last frame: the sectors and the spawned models it could not take in, and the
/// effects whose particles stopped playing.
#[derive(Default)]
pub struct RenderSceneFeedback {
  pub sector_failures: Vec<RenderSectorFailure>,
  pub model_failures: Vec<RenderModelFailure>,
  pub finished_effects: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
}
