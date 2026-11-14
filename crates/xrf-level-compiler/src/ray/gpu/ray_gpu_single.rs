use xrf_geometry::Ray;

/// A single ray toward a light at a position, where its answer goes, and the face it passes (`Single`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RayGpuSingle {
  origin: [f32; 3],
  range: f32,
  direction: [f32; 3],
  index: u32,
  skip: u32,
  _pad: [u32; 3],
}

impl RayGpuSingle {
  /// `ray`, its answer at `index`, passing the face of skip code `skip`
  /// ([`crate::ray::gpu::ray_gpu_tracer::RayGpuTracer::get_skip`]).
  pub(crate) fn new(ray: &Ray, index: u32, skip: u32) -> Self {
    Self {
      origin: ray.origin.to_array(),
      range: ray.range,
      direction: ray.direction.to_array(),
      index,
      skip,
      _pad: [0; 3],
    }
  }
}
