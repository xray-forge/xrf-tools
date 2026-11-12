use xrf_geometry::BvhTracer;
use xrf_level::{LevelDetailsFile, LevelLightsFile};

use crate::shadow::shadow_materials::ShadowMaterials;

/// What [`crate::DetailLighter::light`] lights a level's details from, each already read.
#[derive(Clone, Copy)]
pub struct DetailLightRequest<'a> {
  /// The details as the Build exported them, `build.details`: the slots to light, their light zeroed.
  pub details: &'a LevelDetailsFile,
  /// xrLC's ray model, `build.cform`'s faces (`BuildCformFile::build_tracer`).
  pub tracer: &'a BvhTracer,
  /// How each of the ray model's faces meets a light ray.
  pub shadows: &'a ShadowMaterials,
  /// xrLC's light lists, `build.lights`.
  pub lights: &'a LevelLightsFile,
}
