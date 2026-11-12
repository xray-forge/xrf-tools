use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobScope};
use xrf_level::{LevelDetailsFile, LevelDetailsSlot, LevelLightRole};

use crate::detail::detail_light_request::DetailLightRequest;
use crate::detail::detail_light_source::DetailLightSource;
use crate::detail::detail_lighting::DetailLighting;
use crate::detail::detail_slot_counts::DetailSlotCounts;
use crate::detail::detail_slot_fate::DetailSlotFate;
use crate::detail::detail_slot_lighter::DetailSlotLighter;

/// Lights a level's details as xrDO_Light does: each slot's bare corners cleared, then each planted slot touching the
/// ray model lit from 225 points across it by every light that reaches it, the light scaled by what each shadow ray
/// crosses. Unlike the SDK's lighter, every shadow ray is a full query, never a test of the face that last blocked the
/// light first, so the result is the same whatever the order and the thread count.
pub struct DetailLighter;

impl DetailLighter {
  /// What a job reports the lighting under: the grid's slots.
  pub const PROGRESS_SLOTS: &'static str = "slots";

  /// The request's details lit, the slots shared over the installed pool.
  ///
  /// # Errors
  ///
  /// Returns [`xrf_error::XrfError::Cancelled`] when the job is cancelled, the lighting then left unfinished.
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
    let fates: Vec<DetailSlotFate> = details
      .slots
      .par_iter()
      .enumerate()
      .map(|(cell, words)| {
        job.check_cancelled()?;

        let fate: DetailSlotFate = Self::get_fate(&lighter, cell, words);

        scope.advance();

        Ok(fate)
      })
      .collect::<XrfResult<_>>()?;
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
    })
  }

  /// What the lighter does with the slot of `words` in grid cell `cell`.
  fn get_fate(lighter: &DetailSlotLighter<'_>, cell: usize, words: &[u32; LevelDetailsSlot::WORDS]) -> DetailSlotFate {
    if !LevelDetailsSlot::of(words).is_planted() {
      return DetailSlotFate::Unplanted;
    }

    let slot: LevelDetailsSlot = LevelDetailsSlot::of(&LevelDetailsSlot::with_bare_corners_cleared(*words));

    if !slot.is_planted() {
      return DetailSlotFate::Emptied;
    }

    lighter
      .light(cell, &slot)
      .map_or(DetailSlotFate::Untouched, DetailSlotFate::Lit)
  }
}
