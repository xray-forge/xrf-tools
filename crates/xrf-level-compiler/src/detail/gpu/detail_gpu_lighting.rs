use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobScope};
use xrf_renderer_core::{RayQueryAsk, RayQueryDevice, RayQueryGpu};

use crate::detail::detail_light_request::DetailLightRequest;
use crate::detail::detail_lighter::DetailLighter;
use crate::detail::detail_slot_fate::DetailSlotFate;
use crate::detail::detail_slot_lighter::DetailSlotLighter;
use crate::detail::detail_slot_stage::DetailSlotStage;
use crate::detail::gpu::detail_gpu_batch::DetailGpuBatch;
use crate::detail::gpu::detail_gpu_tracer::DetailGpuTracer;

/// The detail lighter with its shadow rays made and traced on the GPU: slots planned on the CPU's threads a few at a
/// time until enough rays wait, those rays traced in one go, then those slots finished on the CPU's threads.
pub(crate) struct DetailGpuLighting {
  tracer: DetailGpuTracer,
}

impl DetailGpuLighting {
  /// Slots planned together, then planned again until enough rays wait.
  const PLANNED: usize = 64;

  /// Rays traced together at least, unless the slots run out first.
  const RAYS: usize = 1 << 23;

  /// Packets traced together at most before another batch: a dispatch's workgroups, less room for one more plan's.
  const PACKETS: usize = DetailGpuTracer::MOST_WORKGROUPS / 2;

  /// The request's GPU started, the ray model on it, before any slot is lit.
  ///
  /// # Errors
  ///
  /// Returns an error when no GPU traces rays or the GPU refuses the ray model.
  pub(crate) fn start(request: &DetailLightRequest<'_>) -> XrfResult<Self> {
    let device: RayQueryDevice = RayQueryDevice::request(RayQueryAsk {
      dxc: request.dxc,
      gpu: request.gpu.map(|gpu| RayQueryGpu {
        vendor: gpu.vendor,
        device: gpu.device,
      }),
    })?;

    Ok(Self {
      tracer: DetailGpuTracer::new(device, request.tracer, request.shadows)?,
    })
  }

  /// Every slot's fate, and the name of the GPU that traced.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch, or the job is cancelled.
  pub(crate) fn light(
    &self,
    request: &DetailLightRequest<'_>,
    lighter: &DetailSlotLighter<'_>,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<(Vec<DetailSlotFate>, String)> {
    let tracer: &DetailGpuTracer = &self.tracer;
    let slots: &[[u32; 4]] = &request.details.slots;
    let mut fates: Vec<DetailSlotFate> = Vec::with_capacity(slots.len());
    let mut waiting: Vec<DetailSlotStage> = Vec::new();
    let mut batch: DetailGpuBatch = DetailGpuBatch::default();
    let mut firsts: Vec<usize> = Vec::new();
    let mut next: usize = 0;

    while next < slots.len() {
      job.check_cancelled()?;

      let end: usize = (next + Self::PLANNED).min(slots.len());
      let planned: Vec<DetailSlotStage> = (next..end)
        .into_par_iter()
        .map(|cell| DetailLighter::plan(lighter, cell, &slots[cell]))
        .collect();

      for stage in planned {
        firsts.push(match &stage {
          DetailSlotStage::Planned(plan) => batch.push(plan),
          DetailSlotStage::Met(_) => 0,
        });
        waiting.push(stage);
      }

      next = end;

      if batch.ray_count >= Self::RAYS || batch.packets.len() >= Self::PACKETS || next == slots.len() {
        let transmittances: Vec<f32> = tracer.trace(&batch)?;
        let finished: Vec<DetailSlotFate> = waiting
          .par_drain(..)
          .zip(firsts.par_drain(..))
          .map(|(stage, first)| match stage {
            DetailSlotStage::Met(fate) => fate,
            DetailSlotStage::Planned(plan) => {
              DetailSlotFate::Lit(lighter.finish(&plan, &transmittances[first..first + plan.ray_count]))
            }
          })
          .collect();

        scope.advance_by(finished.len() as u64);
        fates.extend(finished);
        batch = DetailGpuBatch::default();
      }
    }

    Ok((fates, tracer.get_name().to_owned()))
  }
}
