use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobScope};
use xrf_level::LevelDetailsFile;

use crate::detail::detail_lighter::DetailLighter;
use crate::detail::detail_slot_fate::DetailSlotFate;
use crate::detail::detail_slot_lighter::DetailSlotLighter;
use crate::detail::detail_slot_stage::DetailSlotStage;
use crate::ray::gpu::ray_gpu_batch::RayGpuBatch;
use crate::ray::gpu::ray_gpu_tracer::RayGpuTracer;

/// The detail lighter with its shadow rays made and traced on a started GPU: slots planned on the CPU's threads a few
/// at a time until enough rays wait, those rays traced in one go, then those slots finished on the CPU's threads.
pub(crate) struct DetailGpuLighting<'a> {
  /// The GPU holding the lighter's ray scene, started by `RayDevice`.
  pub(crate) tracer: &'a RayGpuTracer,
}

impl DetailGpuLighting<'_> {
  /// Slots planned together, then planned again until enough rays wait.
  const PLANNED: usize = 64;

  /// Every slot's fate of `details`.
  ///
  /// # Errors
  ///
  /// Returns an error when the GPU refuses a batch, or the job is cancelled.
  pub(crate) fn light(
    &self,
    details: &LevelDetailsFile,
    lighter: &DetailSlotLighter<'_>,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<Vec<DetailSlotFate>> {
    let tracer: &RayGpuTracer = self.tracer;
    let slots: &[[u32; 4]] = &details.slots;
    let mut fates: Vec<DetailSlotFate> = Vec::with_capacity(slots.len());
    let mut waiting: Vec<DetailSlotStage> = Vec::new();
    let mut batch: RayGpuBatch = RayGpuBatch::default();
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
          DetailSlotStage::Planned(plan) => batch.push(plan.as_ref(), |face| tracer.get_skip(face)),
          DetailSlotStage::Met(_) => 0,
        });
        waiting.push(stage);
      }

      next = end;

      // Traced once no room is left for one more packet or single ray, or enough rays wait.
      if batch.is_full(1, 1) || next == slots.len() {
        let transmittances: Vec<f32> = tracer.trace(&batch, DetailSlotLighter::get_rules())?;
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
        batch = RayGpuBatch::default();
      }
    }

    Ok(fates)
  }
}
