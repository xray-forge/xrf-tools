/// Result of verifying round-trip of a single omf file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepackOmfOutcome {
  Identical,
  Mismatched,
  Failed,
}

/// Aggregated outcome of verifying round-trip of many omf files.
#[derive(Default)]
pub struct RepackOmfStatistics {
  checked: u32,
  failed: u32,
  mismatched: u32,
}

impl RepackOmfStatistics {
  /// Account single verified file in the aggregated statistics.
  pub fn register(&mut self, outcome: RepackOmfOutcome) {
    self.checked += 1;

    match outcome {
      RepackOmfOutcome::Identical => {}
      RepackOmfOutcome::Mismatched => self.mismatched += 1,
      RepackOmfOutcome::Failed => self.failed += 1,
    }
  }

  /// Count of files verified so far.
  pub fn checked(&self) -> u32 {
    self.checked
  }

  /// Count of files that were read but serialized into different bytes.
  pub fn mismatched(&self) -> u32 {
    self.mismatched
  }

  /// Count of files that could not be read or written.
  pub fn failed(&self) -> u32 {
    self.failed
  }

  /// Count of files that serialized back into identical bytes.
  pub fn identical(&self) -> u32 {
    self.checked - self.mismatched - self.failed
  }

  /// Whether every verified file serialized back into identical bytes.
  pub fn is_valid(&self) -> bool {
    self.mismatched == 0 && self.failed == 0
  }
}

#[cfg(test)]
mod tests {
  use crate::commands::omf::repack::statistics::{RepackOmfOutcome, RepackOmfStatistics};

  #[test]
  fn test_empty_statistics_are_valid() {
    let statistics: RepackOmfStatistics = RepackOmfStatistics::default();

    assert_eq!(statistics.checked(), 0);
    assert_eq!(statistics.identical(), 0);
    assert!(statistics.is_valid());
  }

  #[test]
  fn test_statistics_count_identical_outcomes() {
    let mut statistics: RepackOmfStatistics = RepackOmfStatistics::default();

    statistics.register(RepackOmfOutcome::Identical);
    statistics.register(RepackOmfOutcome::Identical);

    assert_eq!(statistics.checked(), 2);
    assert_eq!(statistics.identical(), 2);
    assert!(statistics.is_valid());
  }

  #[test]
  fn test_statistics_count_mismatched_and_failed_outcomes() {
    let mut statistics: RepackOmfStatistics = RepackOmfStatistics::default();

    statistics.register(RepackOmfOutcome::Identical);
    statistics.register(RepackOmfOutcome::Mismatched);
    statistics.register(RepackOmfOutcome::Failed);

    assert_eq!(statistics.checked(), 3);
    assert_eq!(statistics.mismatched(), 1);
    assert_eq!(statistics.failed(), 1);
    assert_eq!(statistics.identical(), 1);
    assert!(!statistics.is_valid());
  }
}
