use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobScope};
use xrf_level::LevelDetailsFile;

use crate::detail::detail_lighter::DetailLighter;
use crate::detail::detail_slot_fate::DetailSlotFate;
use crate::detail::detail_slot_lighter::DetailSlotLighter;
use crate::detail::detail_slot_stage::DetailSlotStage;
use crate::detail::gpu::detail_gpu_waiting::DetailGpuWaiting;
use crate::ray::gpu::ray_gpu_batch::RayGpuBatch;
use crate::ray::gpu::ray_gpu_tracer::RayGpuTracer;

/// The detail lighter with its shadow rays traced on a started GPU: slots planned on CPU threads until enough rays
/// wait, those rays traced in one dispatch, then those slots finished on CPU threads.
pub(crate) struct DetailGpuLighting<'a> {
  /// The GPU holding the lighter's ray scene, started by `RayDevice`.
  pub(crate) tracer: &'a RayGpuTracer,
}

impl DetailGpuLighting<'_> {
  /// Slots planned together, then planned again until enough rays wait.
  const PLANNED: usize = 64;

  /// Every slot's fate.
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
    let slots: &[[u32; 4]] = &details.slots;
    let mut fates: Vec<DetailSlotFate> = Vec::with_capacity(slots.len());
    let mut waiting: DetailGpuWaiting = DetailGpuWaiting::default();
    let mut next: usize = 0;

    while next < slots.len() {
      job.check_cancelled()?;

      let end: usize = (next + Self::PLANNED).min(slots.len());
      let planned: Vec<DetailSlotStage> = (next..end)
        .into_par_iter()
        .map(|cell| DetailLighter::plan(lighter, cell, &slots[cell]))
        .collect();

      // Slot by slot, so a slot whose rays would overflow the dispatch goes in the next.
      for stage in planned {
        let first: usize = match &stage {
          DetailSlotStage::Planned(plan) => {
            if !waiting.stages.is_empty() && waiting.batch.is_full_for(plan.as_ref()) {
              fates.extend(self.trace(&mut waiting, lighter, scope)?);
            }

            waiting.batch.push(plan.as_ref(), |face| self.tracer.get_skip(face))
          }
          DetailSlotStage::Met(_) => 0,
        };

        waiting.firsts.push(first);
        waiting.stages.push(stage);
      }

      next = end;

      // Traced once not one more packet or single ray fits, or the slots run out.
      if waiting.batch.is_full(1, 1) || next == slots.len() {
        fates.extend(self.trace(&mut waiting, lighter, scope)?);
      }
    }

    Ok(fates)
  }

  /// The fates of the waiting slots, their rays traced, leaving nothing waiting.
  fn trace(
    &self,
    waiting: &mut DetailGpuWaiting,
    lighter: &DetailSlotLighter<'_>,
    scope: &JobScope,
  ) -> XrfResult<Vec<DetailSlotFate>> {
    let batch: RayGpuBatch = std::mem::take(&mut waiting.batch);
    let transmittances: Vec<f32> = self.tracer.trace(&batch, DetailSlotLighter::get_rules())?;
    let finished: Vec<DetailSlotFate> = waiting
      .stages
      .par_drain(..)
      .zip(waiting.firsts.par_drain(..))
      .map(|(stage, first)| match stage {
        DetailSlotStage::Met(fate) => fate,
        DetailSlotStage::Planned(plan) => {
          DetailSlotFate::Lit(lighter.finish(&plan, &transmittances[first..first + plan.ray_count]))
        }
      })
      .collect();

    scope.advance_by(finished.len() as u64);

    Ok(finished)
  }
}
