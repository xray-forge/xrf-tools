use xrf_renderer_core::ProxyHandle;

use crate::host::level::particles::placed_effect::PlacedEffect;
use crate::host::world::feedback::render_model_failure::RenderModelFailure;
use crate::host::world::feedback::render_sector_failure::RenderSectorFailure;
use crate::scene::level::particles::placed::particle_emitter_proxy::ParticleEmitterProxy;

/// What a scene tells the world of the last frame: the sectors and the spawned models it could not take in, the effects
/// whose particles stopped playing, and whether the level has stood ready.
#[derive(Default)]
pub struct RenderSceneFeedback {
  pub sector_failures: Vec<RenderSectorFailure>,
  pub model_failures: Vec<RenderModelFailure>,
  pub finished_effects: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
  /// Whether everything the level opens with has been resident once since its scene was made, so it drew as it will:
  /// from the moment its load reports it ready on.
  pub has_stood_ready: bool,
}
