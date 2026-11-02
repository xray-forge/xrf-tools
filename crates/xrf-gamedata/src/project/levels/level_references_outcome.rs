use crate::Finding;

/// Findings and counters from resolving what a level's geometry references.
#[derive(Default)]
pub(crate) struct LevelReferencesOutcome {
  pub(crate) findings: Vec<Finding>,
  pub(crate) checked_count: u32,
  pub(crate) invalid_count: u32,
}
