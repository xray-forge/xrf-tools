use xrf_geometry::BvhTracer;

use crate::shadow::shadow_materials::ShadowMaterials;

/// What a compiler traces its rays through: a ray model in a tracer, and how each of its faces meets a ray.
#[derive(Clone, Copy)]
pub(crate) struct RayScene<'a> {
  pub tracer: &'a BvhTracer,
  /// One per face of `tracer`.
  pub shadows: &'a ShadowMaterials,
}
