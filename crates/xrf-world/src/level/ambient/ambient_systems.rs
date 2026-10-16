use xrf_engine_target::XrayEngine;
use xrf_particles::ParticleLibrary;
use xrf_renderer::ParticleEmitterProxy;
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};

/// What an update of the effects reads besides the frame's ambients: the effects and groups the level has and the
/// engine they play by, the emitters the scene's posts name, and the emitters whose particles died.
pub struct AmbientSystems<'a> {
  pub library: &'a ParticleLibrary,
  pub engine: XrayEngine,
  pub emitters: &'a mut ProxyAllocator<ParticleEmitterProxy>,
  pub finished: &'a [ProxyHandle<ParticleEmitterProxy>],
}
