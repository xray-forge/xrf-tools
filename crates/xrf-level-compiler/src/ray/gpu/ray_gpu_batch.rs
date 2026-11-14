use crate::ray::gpu::ray_gpu_origin::RayGpuOrigin;
use crate::ray::gpu::ray_gpu_packet::RayGpuPacket;
use crate::ray::gpu::ray_gpu_plan::RayGpuPlan;
use crate::ray::gpu::ray_gpu_single::RayGpuSingle;
use crate::ray::gpu::ray_gpu_tracer::RayGpuTracer;
use crate::ray::ray_group::RayGroup;

/// The rays of several plans as the GPU makes and traces them: their points' origins, each with the skip code of the
/// face its rays pass, their packets and their single rays, every ray numbered in one run, a plan's after the plan
/// before.
#[derive(Default)]
pub(crate) struct RayGpuBatch {
  pub(crate) packets: Vec<RayGpuPacket>,
  pub(crate) origins: Vec<RayGpuOrigin>,
  pub(crate) singles: Vec<RayGpuSingle>,
  pub(crate) ray_count: usize,
}

impl RayGpuBatch {
  /// Rays from which a batch is traced, unless its plans run out first.
  const FULL_RAYS: usize = 1 << 23;

  /// Packets a batch holds at most: a dispatch's workgroups, less room for one more plan's.
  const FULL_PACKETS: usize = RayGpuTracer::MOST_WORKGROUPS / 2;

  /// Adds `plan`'s rays, each point's face named by its skip code `skip` gives, answering the number its first ray
  /// takes.
  pub(crate) fn push(&mut self, plan: &impl RayGpuPlan, skip: impl Fn(Option<u32>) -> u32) -> usize {
    let first_ray: usize = self.ray_count;
    let first_origin: u32 = self.origins.len() as u32;
    let codes: Vec<u32> = (0..plan.get_point_count())
      .map(|point| skip(plan.get_skip(point)))
      .collect();

    self.origins.extend(
      codes
        .iter()
        .enumerate()
        .map(|(point, code)| RayGpuOrigin::new(plan.get_origin(point), plan.get_point(point), *code)),
    );

    for group in plan.get_groups() {
      match group {
        RayGroup::Packet {
          toward,
          lift,
          range,
          facing,
          first,
        } => self.packets.push(RayGpuPacket::new_directional(
          *toward,
          *lift,
          *range,
          *facing,
          first_origin,
          (first_ray + *first as usize) as u32,
        )),
        RayGroup::Lamp {
          position,
          facing,
          first,
        } => self.packets.push(RayGpuPacket::new_lamp(
          *position,
          *facing,
          first_origin,
          (first_ray + *first as usize) as u32,
        )),
        RayGroup::Single { ray, index, point } => self.singles.push(RayGpuSingle::new(
          ray,
          (first_ray + *index as usize) as u32,
          codes[*point as usize],
        )),
      }
    }

    self.ray_count += plan.get_ray_count();

    first_ray
  }

  /// Whether the batch is to be traced before it takes `next_packets` more packets and `next_singles` more single
  /// rays: it would hold more packets or single rays than a dispatch takes, or holds enough rays already.
  pub(crate) fn is_full(&self, next_packets: usize, next_singles: usize) -> bool {
    self.packets.len() + next_packets > Self::FULL_PACKETS
      || self.singles.len() + next_singles > RayGpuTracer::MOST_SINGLES
      || self.ray_count >= Self::FULL_RAYS
  }
}
