use crate::light::light_plan::LightPlan;
use crate::light::light_ray_group::LightRayGroup;
use crate::ray::gpu::ray_gpu_records::{GpuOrigin, GpuPacket, GpuSingle};

/// The rays of several plans as the GPU makes and traces them: their points' origins, each with the skip code of the
/// face its rays pass, their packets and their single rays, every ray numbered in one run, a plan's after the plan
/// before.
#[derive(Default)]
pub(crate) struct RayGpuBatch {
  pub(crate) packets: Vec<GpuPacket>,
  pub(crate) origins: Vec<GpuOrigin>,
  pub(crate) singles: Vec<GpuSingle>,
  pub(crate) ray_count: usize,
}

impl RayGpuBatch {
  /// Adds `plan`'s rays, each point's face named by its skip code `skip` gives, answering the number its first ray
  /// takes.
  pub(crate) fn push(&mut self, plan: &LightPlan, skip: impl Fn(Option<u32>) -> u32) -> usize {
    let first_ray: usize = self.ray_count;
    let first_origin: u32 = self.origins.len() as u32;
    let codes: Vec<u32> = plan.skips.iter().map(|face| skip(*face)).collect();

    self.origins.extend(
      plan
        .points
        .iter()
        .zip(&codes)
        .map(|(point, code)| GpuOrigin::new(point, *code)),
    );

    for group in &plan.groups {
      match group {
        LightRayGroup::Packet {
          toward,
          lift,
          facing,
          first,
        } => self.packets.push(GpuPacket::new_directional(
          *toward,
          *lift,
          *facing,
          first_origin,
          (first_ray + *first as usize) as u32,
        )),
        LightRayGroup::Lamp {
          position,
          facing,
          first,
        } => self.packets.push(GpuPacket::new_lamp(
          *position,
          *facing,
          first_origin,
          (first_ray + *first as usize) as u32,
        )),
        LightRayGroup::Single { ray, index, point } => self.singles.push(GpuSingle::new(
          ray,
          (first_ray + *index as usize) as u32,
          codes[*point as usize],
        )),
      }
    }

    self.ray_count += plan.ray_count;

    first_ray
  }
}
