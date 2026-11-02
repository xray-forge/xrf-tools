use crate::Finding;

/// What a sweep found over its items: how many it checked, how many raised a finding, and the findings, ordered by the
/// asset they are about.
pub(crate) struct GamedataAssetSweepOutcome {
  pub(crate) checked_count: u32,
  pub(crate) invalid_count: u32,
  pub(crate) findings: Vec<Finding>,
}
