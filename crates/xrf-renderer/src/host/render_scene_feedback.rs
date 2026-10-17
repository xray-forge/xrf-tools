use xrf_renderer_core::ProxyHandle;

use crate::host::render_model_failure::RenderModelFailure;
use crate::host::render_sector_failure::RenderSectorFailure;
use crate::scene::level::particle_emitter_proxy::ParticleEmitterProxy;
use crate::scene::level::placed_effect::PlacedEffect;

/// What a scene tells the world of the last frame: the sectors and the spawned models it could not take in, and the
/// effects whose particles stopped playing.
#[derive(Default)]
pub struct RenderSceneFeedback {
  pub sector_failures: Vec<RenderSectorFailure>,
  pub model_failures: Vec<RenderModelFailure>,
  pub finished_effects: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
}
