use crate::detail::detail_slot_plan::{DetailRayGroup, DetailSlotPlan};
use crate::detail::gpu::detail_gpu_records::{GpuPacket, GpuSingle};

/// The rays of several slots' plans as the GPU makes and traces them: their points' origins, their packets and their
/// single rays, every ray numbered in one run, a slot's after the slot before.
#[derive(Default)]
pub(crate) struct DetailGpuBatch {
  pub(crate) packets: Vec<GpuPacket>,
  pub(crate) origins: Vec<[f32; 4]>,
  pub(crate) singles: Vec<GpuSingle>,
  pub(crate) ray_count: usize,
}

impl DetailGpuBatch {
  /// Adds `plan`'s rays, answering the number its first ray takes.
  pub(crate) fn push(&mut self, plan: &DetailSlotPlan) -> usize {
    let first_ray: usize = self.ray_count;
    let first_origin: u32 = self.origins.len() as u32;

    self
      .origins
      .extend(plan.origins.iter().map(|origin| origin.extend(0.0).to_array()));

    for group in &plan.groups {
      match group {
        DetailRayGroup::Packet {
          toward,
          lift,
          facing,
          first,
        } => self.packets.push(GpuPacket::new(
          *toward,
          *lift,
          *facing,
          first_origin,
          (first_ray + *first as usize) as u32,
        )),
        DetailRayGroup::Single { ray, index } => self
          .singles
          .push(GpuSingle::new(ray, (first_ray + *index as usize) as u32)),
      }
    }

    self.ray_count += plan.ray_count;

    first_ray
  }
}
