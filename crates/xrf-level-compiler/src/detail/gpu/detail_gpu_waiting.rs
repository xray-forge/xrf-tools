use crate::detail::detail_slot_stage::DetailSlotStage;
use crate::ray::gpu::ray_gpu_batch::RayGpuBatch;

/// Slots planned and waiting for their rays: the batch holding the rays, and each slot's stage with the number its
/// first ray takes in the batch.
#[derive(Default)]
pub(crate) struct DetailGpuWaiting {
  pub(crate) batch: RayGpuBatch,
  pub(crate) stages: Vec<DetailSlotStage>,
  pub(crate) firsts: Vec<usize>,
}
