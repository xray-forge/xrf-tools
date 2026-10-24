use std::collections::HashMap;

use xrf_material::XraySurfaceDescriptor;
use xrf_particles::ParticleLibrary;
use xrf_renderer::RenderParticlePlacement;

use crate::parts::LevelTextureReference;

/// A level's particle systems as the renderer takes them.
pub struct PackedLevelParticles {
  pub library: ParticleLibrary,
  pub placements: Vec<RenderParticlePlacement>,
  /// How each effect a placement can play draws, by its name.
  pub surfaces: HashMap<String, XraySurfaceDescriptor>,
  pub textures: Vec<LevelTextureReference>,
}
