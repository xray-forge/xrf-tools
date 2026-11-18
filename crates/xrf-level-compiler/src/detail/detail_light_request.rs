use xrf_level::{LevelDetailsFile, LevelLightsFile};

use crate::ray::ray_scene::RayScene;
use crate::ray::ray_tracing::RayTracing;

/// What [`crate::detail::detail_lighter::DetailLighter::light`] lights a level's details from, each already read.
#[derive(Clone, Copy)]
pub(crate) struct DetailLightRequest<'a> {
  /// The details as the Build exported them, `build.details`: the slots to light, their light zeroed.
  pub details: &'a LevelDetailsFile,
  /// xrLC's ray model, `build.cform`'s faces (`BuildCformFile::build_tracer`), and how each meets a light ray.
  pub scene: RayScene<'a>,
  /// xrLC's light lists, `build.lights`.
  pub lights: &'a LevelLightsFile,
  /// What traces the shadow rays.
  pub tracing: RayTracing<'a>,
}
