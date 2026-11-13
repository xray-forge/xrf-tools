use xrf_geometry::BvhTracer;

use crate::shadow::shadow_materials::ShadowMaterials;

/// What the cover pass traces its rays against: xrLC's ray model, `build.cform`, in a tracer, and how each of its faces
/// meets a ray.
#[derive(Clone, Copy)]
pub struct AiRayModel<'a> {
  pub tracer: &'a BvhTracer,
  pub shadows: &'a ShadowMaterials,
}
