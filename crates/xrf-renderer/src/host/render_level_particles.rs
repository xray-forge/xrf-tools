use std::collections::HashMap;
use std::sync::Arc;

use xrf_material::XraySurfaceDescriptor;
use xrf_particles::{ParticleCollider, ParticleEngineRules, ParticleLibrary};
use xrf_visual::HemiEstimator;

use crate::host::render_particle_placement::RenderParticlePlacement;

/// A level's particle systems as its loader reads them: the library they play from, how the engine steps them, where
/// they stand, how each effect draws, the surfaces colliding effects meet, and how lit the camera stands, which the
/// weather's ambient effects wait outdoors for.
pub struct RenderLevelParticles {
  pub library: Arc<ParticleLibrary>,
  pub rules: ParticleEngineRules,
  pub placements: Vec<RenderParticlePlacement>,
  /// How each effect any placement can play draws, by its name; its textures are read through
  /// [`crate::RenderAssetSource::read_texture`] by the names its descriptor gives.
  pub surfaces: HashMap<String, XraySurfaceDescriptor>,
  /// The level's collision form, none where it cannot be read, which leaves every effect colliding with nothing.
  pub collider: Option<Arc<dyn ParticleCollider>>,
  /// How the level lights an object standing anywhere, `CROS_impl`'s estimate, none where it cannot be built, which
  /// leaves the camera always outdoors.
  pub hemi: Option<Arc<HemiEstimator>>,
}
