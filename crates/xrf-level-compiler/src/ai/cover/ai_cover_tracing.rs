use crate::ray::ray_scene::RayScene;
use crate::ray::ray_tracing::RayTracing;

/// How the cover pass traces its corner nodes' rays: against what, and on which backend.
#[derive(Clone, Copy)]
pub(crate) struct AiCoverTracing<'a> {
  pub(crate) scene: RayScene<'a>,
  pub(crate) tracing: RayTracing<'a>,
}
