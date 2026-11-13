use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobScope};
use xrf_level::{LevelDetailsFile, LevelDetailsSlot, LevelLightRole};

use crate::detail::detail_light_backend::DetailLightBackend;
use crate::detail::detail_light_request::DetailLightRequest;
use crate::detail::detail_light_source::DetailLightSource;
use crate::detail::detail_lighting::DetailLighting;
use crate::detail::detail_slot_counts::DetailSlotCounts;
use crate::detail::detail_slot_fate::DetailSlotFate;
use crate::detail::detail_slot_lighter::DetailSlotLighter;
use crate::detail::detail_slot_stage::DetailSlotStage;

/// Lights a level's details as xrDO_Light does: each slot's bare corners cleared, then each planted slot touching the
/// ray model lit from 225 points across it by every light that reaches it, the light scaled by what each shadow ray
/// crosses. Unlike the SDK's lighter, every shadow ray is a full query, never a test of the face that last blocked the
/// light first, so the result is the same whatever the order and the thread count.
pub struct DetailLighter;

impl DetailLighter {
  /// What a job reports the lighting under: the grid's slots.
  pub const PROGRESS_SLOTS: &'static str = "slots";

  /// The request's details lit, the slots shared over the installed pool, their shadow rays traced by the request's
  /// backend.
  ///
  /// # Errors
  ///
  /// Returns [`xrf_error::XrfError::Cancelled`] when the job is cancelled, the lighting then left unfinished; and, for
  /// the GPU backend, an error when this build has none, no GPU traces rays, or the GPU fails; `Auto` lights on the
  /// CPU instead when the GPU does not start, but fails with a GPU that fails once started.
  pub fn light(request: &DetailLightRequest<'_>, job: &JobHandle) -> XrfResult<DetailLighting> {
    let details: &LevelDetailsFile = request.details;
    let lights: [Vec<DetailLightSource>; 3] = LevelLightRole::ALL.map(|role| {
      request
        .lights
        .get_role_lights(role)
        .map(DetailLightSource::from)
        .collect()
    });
    let lighter: DetailSlotLighter<'_> = DetailSlotLighter {
      header: &details.header,
      tracer: request.tracer,
      shadows: request.shadows,
      lights: &lights,
    };
    let scope: JobScope = job.enter(Self::PROGRESS_SLOTS, Some(details.slots.len() as u64));
    let (fates, gpu): (Vec<DetailSlotFate>, Option<String>) = match request.backend {
      DetailLightBackend::Cpu => (Self::light_on_cpu(&lighter, details, job, &scope)?, None),
      DetailLightBackend::Gpu | DetailLightBackend::Auto => Self::light_on_gpu(request, &lighter, job, &scope)?,
    };
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
      gpu,
    })
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

  /// Every slot's fate with the shadow rays traced on the GPU, and the GPU's name; for `Auto`, on the CPU when the GPU
  /// does not start. A GPU that fails once started fails the lighting, as the slots it lit are already counted.
  #[cfg(feature = "gpu")]
  fn light_on_gpu(
    request: &DetailLightRequest<'_>,
    lighter: &DetailSlotLighter<'_>,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<(Vec<DetailSlotFate>, Option<String>)> {
    use crate::detail::gpu::detail_gpu_lighting::DetailGpuLighting;

    match DetailGpuLighting::start(request) {
      Ok(gpu) => gpu
        .light(request, lighter, job, scope)
        .map(|(fates, name)| (fates, Some(name))),
      Err(error) if request.backend == DetailLightBackend::Auto => {
        Self::light_on_cpu_instead(&error, lighter, request.details, job, scope)
      }
      Err(error) => Err(error),
    }
  }

  /// Every slot's fate on the CPU for `Auto`; refused for `Gpu`: this build has no GPU backend.
  #[cfg(not(feature = "gpu"))]
  fn light_on_gpu(
    request: &DetailLightRequest<'_>,
    lighter: &DetailSlotLighter<'_>,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<(Vec<DetailSlotFate>, Option<String>)> {
    let error: xrf_error::XrfError = xrf_error::XrfError::new_invalid_error(
      "This build lights details on the CPU alone; its GPU backend is the `gpu` feature",
    );

    if request.backend == DetailLightBackend::Auto {
      Self::light_on_cpu_instead(&error, lighter, request.details, job, scope)
    } else {
      Err(error)
    }
  }

  /// Every slot's fate on the CPU, the GPU having not started.
  fn light_on_cpu_instead(
    error: &xrf_error::XrfError,
    lighter: &DetailSlotLighter<'_>,
    details: &LevelDetailsFile,
    job: &JobHandle,
    scope: &JobScope,
  ) -> XrfResult<(Vec<DetailSlotFate>, Option<String>)> {
    log::info!("Lighting the details on the CPU: {error}");

    Ok((Self::light_on_cpu(lighter, details, job, scope)?, None))
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

    lighter.plan(cell, &slot).map_or(
      DetailSlotStage::Met(DetailSlotFate::Untouched),
      DetailSlotStage::Planned,
    )
  }
}
