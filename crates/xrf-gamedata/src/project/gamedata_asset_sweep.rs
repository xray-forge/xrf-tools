use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_output::{OutputSequence, OutputSlot};

use crate::project::gamedata_asset_sweep_outcome::GamedataAssetSweepOutcome;
use crate::{Finding, GamedataFindingFactory, GamedataProjectVerifyOptions};

/// Verifies many independent items of one kind at once: assets, or the sections that name them.
pub(crate) struct GamedataAssetSweep<'a> {
  options: &'a GamedataProjectVerifyOptions,
}

impl<'a> GamedataAssetSweep<'a> {
  pub(crate) fn new(options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options }
  }

  /// Runs `verify` over every item, handing each the run's options with that item's own output, and gathers what they
  /// found. No item starts after cancellation; the caller checks it once the sweep returns.
  ///
  /// # Errors
  ///
  /// Returns an error when there are more items than a report can count.
  pub(crate) fn run<T, F>(&self, items: &[T], verify: F) -> XrfResult<GamedataAssetSweepOutcome>
  where
    T: Sync,
    F: Fn(&GamedataProjectVerifyOptions, &T) -> Vec<Finding> + Sync,
  {
    let sequence: OutputSequence = OutputSequence::new(&self.options.output, items.len());

    let findings_per_item: Vec<Vec<Finding>> = items
      .par_iter()
      .enumerate()
      .filter(|_| !self.options.job.is_cancelled())
      .map(|(index, item)| {
        // Held until the item is done: the slot releases its position when dropped.
        let slot: OutputSlot = sequence.new_slot(index);

        verify(&self.options.with_output(slot.get_output().clone()), item)
      })
      .collect();

    let invalid_count: usize = findings_per_item.iter().filter(|findings| !findings.is_empty()).count();
    let mut findings: Vec<Finding> = findings_per_item.into_iter().flatten().collect();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);

    Ok(GamedataAssetSweepOutcome {
      checked_count: Self::to_count(items.len())?,
      invalid_count: Self::to_count(invalid_count)?,
      findings,
    })
  }

  fn to_count(count: usize) -> XrfResult<u32> {
    u32::try_from(count).map_err(|_| XrfError::new_verify_error("Item count exceeds the supported result range"))
  }
}
