use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobScope};
use xrf_level::{LevelDetailsFile, LevelDetailsSlot};

use crate::detail::detail_light_request::DetailLightRequest;
use crate::detail::detail_lighting::DetailLighting;
use crate::detail::detail_slot_counts::DetailSlotCounts;
use crate::detail::detail_slot_fate::DetailSlotFate;
use crate::detail::detail_slot_lighter::DetailSlotLighter;
use crate::detail::detail_slot_stage::DetailSlotStage;
use crate::light::source::light_sources::LightSources;
use crate::ray::ray_device::RayDevice;

/// Lights a level's details as xrDO_Light does: each slot's bare corners cleared, then each planted slot touching the
/// ray model lit from 225 points across it by every light that reaches it, the light scaled by what each shadow ray
/// crosses. Unlike xrDO_Light, every shadow ray is a full query, never a test of the face that last blocked the light
/// first, so the result is the same whatever the order and the thread count.
pub struct DetailLighter;

impl DetailLighter {
  /// What a job reports the lighting under: the grid's slots.
  pub(crate) const PROGRESS_SLOTS: &'static str = "slots";

  /// The request's details lit, the slots shared over the installed pool, their shadow rays traced by the request's
  /// backend.
  ///
  /// # Errors
  ///
  /// Returns [`xrf_error::XrfError::Cancelled`] when the job is cancelled, the lighting then left unfinished; and, for
  /// the GPU backend, an error when this build has none, no GPU traces rays, or the GPU fails; `Auto` lights on the
  /// CPU instead when the GPU does not start, but fails with a GPU that fails once started, as the slots it lit are
  /// already counted.
  pub fn light(request: &DetailLightRequest<'_>, job: &JobHandle) -> XrfResult<DetailLighting> {
    let details: &LevelDetailsFile = request.details;
    let lights: LightSources = LightSources::from_file(request.lights);
    let lighter: DetailSlotLighter<'_> = DetailSlotLighter {
      header: &details.header,
      scene: request.scene,
      lights: &lights,
    };
    let scope: JobScope = job.enter(Self::PROGRESS_SLOTS, Some(details.slots.len() as u64));
    let device: RayDevice = RayDevice::start(&request.tracing, request.scene, "the details' shadow rays")?;
    let fates: Vec<DetailSlotFate> = Self::light_on(&device, &lighter, details, job, &scope)?;
    let mut counts: DetailSlotCounts = DetailSlotCounts::default();
    let slots: Vec<[u32; LevelDetailsSlot::WORDS]> = details
      .slots
      .iter()
      .zip(&fates)
      .map(|(words, fate)| {
        let cleared: [u32; LevelDetailsSlot::WORDS] = LevelDetailsSlot::with_bare_corners_cleared(*words);

        match fate {
          DetailSlotFate::Unplanted => counts.unplanted += 1,
          DetailSlotFate::Emptied => counts.emptied += 1,
          DetailSlotFate::Untouched => counts.untouched += 1,
          DetailSlotFate::Lit(light) => {
            counts.lit += 1;

            return LevelDetailsSlot::with_light(cleared, *light);
          }
        }

        cleared
      })
      .collect();

    Ok(DetailLighting {
      details: LevelDetailsFile {
        header: details.header.clone(),
        objects: details.objects.clone(),
        slots,
      },
      counts,
      gpu: device.get_name(),
    })
  }

  /// Every slot's fate, its shadow rays traced on `device`.
  fn light_on(
    device: &RayDevice,
    lighter: &DetailSlotLighter<'_>,
    details: &LevelDetailsFile,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<Vec<DetailSlotFate>> {
    #[cfg(feature = "gpu")]
    if let Some(tracer) = device.get_gpu() {
      return crate::detail::gpu::detail_gpu_lighting::DetailGpuLighting { tracer }.light(details, lighter, job, scope);
    }

    #[cfg(not(feature = "gpu"))]
    let _ = device;

    Self::light_on_cpu(lighter, details, job, scope)
  }

  /// Every slot's fate, each lit whole on a CPU thread: planned, its rays traced as packets, finished.
  fn light_on_cpu(
    lighter: &DetailSlotLighter<'_>,
    details: &LevelDetailsFile,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<Vec<DetailSlotFate>> {
    details
      .slots
      .par_iter()
      .enumerate()
      .map(|(cell, words)| {
        job.check_cancelled()?;

        let fate: DetailSlotFate = match Self::plan(lighter, cell, words) {
          DetailSlotStage::Met(fate) => fate,
          DetailSlotStage::Planned(plan) => DetailSlotFate::Lit(lighter.finish(&plan, &lighter.trace(&plan))),
        };

        scope.advance();

        Ok(fate)
      })
      .collect()
  }

  /// What the lighter does with the slot of `words` in grid cell `cell`: its fate where that needs no ray, else its
  /// plan.
  pub(crate) fn plan(
    lighter: &DetailSlotLighter<'_>,
    cell: usize,
    words: &[u32; LevelDetailsSlot::WORDS],
  ) -> DetailSlotStage {
    if !LevelDetailsSlot::of(words).is_planted() {
      return DetailSlotStage::Met(DetailSlotFate::Unplanted);
    }

    let slot: LevelDetailsSlot = LevelDetailsSlot::of(&LevelDetailsSlot::with_bare_corners_cleared(*words));

    if !slot.is_planted() {
      return DetailSlotStage::Met(DetailSlotFate::Emptied);
    }

    lighter
      .plan(cell, &slot)
      .map_or(DetailSlotStage::Met(DetailSlotFate::Untouched), |plan| {
        DetailSlotStage::Planned(Box::new(plan))
      })
  }
}
