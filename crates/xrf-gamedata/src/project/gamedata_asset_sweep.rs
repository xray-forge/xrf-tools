use rayon::prelude::*;
use xrf_error::XrfResult;
use xrf_utils::narrow_format_size;

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

  /// Runs `verify` over every item and gathers what they found, ordered so the outcome does not depend on how the
  /// items were scheduled. No item starts after cancellation; the caller checks it once the sweep returns.
  ///
  /// # Errors
  ///
  /// Returns an error when there are more items than a report can count.
  pub(crate) fn run<T, F>(&self, items: &[T], verify: F) -> XrfResult<GamedataAssetSweepOutcome>
  where
    T: Sync,
    F: Fn(&T) -> Vec<Finding> + Sync,
  {
    let findings_per_item: Vec<Vec<Finding>> = items
      .par_iter()
      .filter(|_| !self.options.job.is_cancelled())
      .map(&verify)
      .collect();

    let invalid_count: usize = findings_per_item.iter().filter(|findings| !findings.is_empty()).count();
    let mut findings: Vec<Finding> = findings_per_item.into_iter().flatten().collect();

    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);

    Ok(GamedataAssetSweepOutcome {
      checked_count: narrow_format_size(items.len(), "Checked assets")?,
      invalid_count: narrow_format_size(invalid_count, "Invalid assets")?,
      findings,
    })
  }
}
