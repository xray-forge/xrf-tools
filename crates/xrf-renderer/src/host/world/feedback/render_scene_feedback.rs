use xrf_renderer_core::ProxyHandle;

use crate::host::level::particles::placed_effect::PlacedEffect;
use crate::host::world::feedback::render_model_failure::RenderModelFailure;
use crate::host::world::feedback::render_sector_failure::RenderSectorFailure;
use crate::scene::level::particles::placed::particle_emitter_proxy::ParticleEmitterProxy;

/// A scene's report to the world since the last: failed sectors and models, finished effects, and readiness.
#[derive(Default)]
pub struct RenderSceneFeedback {
  pub sector_failures: Vec<RenderSectorFailure>,
  pub model_failures: Vec<RenderModelFailure>,
  pub finished_effects: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
  /// Whether everything the level opens with has been resident at least once since its scene was made.
  pub has_stood_ready: bool,
}
