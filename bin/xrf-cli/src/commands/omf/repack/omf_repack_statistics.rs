use crate::commands::omf::repack::OmfRepackOutcome;

/// How the round trips of many omf files came out, counted.
#[derive(Default)]
pub struct OmfRepackStatistics {
  checked: u32,
  failed: u32,
  mismatched: u32,
}

impl OmfRepackStatistics {
  /// Counts one file's round trip.
  pub fn register(&mut self, outcome: OmfRepackOutcome) {
    self.checked += 1;

    match outcome {
      OmfRepackOutcome::Identical => {}
      OmfRepackOutcome::Mismatched => self.mismatched += 1,
      OmfRepackOutcome::Failed => self.failed += 1,
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
  use crate::commands::omf::repack::{OmfRepackOutcome, OmfRepackStatistics};

  #[test]
  fn empty_statistics_are_valid() {
    let statistics: OmfRepackStatistics = OmfRepackStatistics::default();

    assert_eq!(statistics.checked(), 0);
    assert_eq!(statistics.identical(), 0);
    assert!(statistics.is_valid());
  }

  #[test]
  fn statistics_count_identical_outcomes() {
    let mut statistics: OmfRepackStatistics = OmfRepackStatistics::default();

    statistics.register(OmfRepackOutcome::Identical);
    statistics.register(OmfRepackOutcome::Identical);

    assert_eq!(statistics.checked(), 2);
    assert_eq!(statistics.identical(), 2);
    assert!(statistics.is_valid());
  }

  #[test]
  fn statistics_count_mismatched_and_failed_outcomes() {
    let mut statistics: OmfRepackStatistics = OmfRepackStatistics::default();

    statistics.register(OmfRepackOutcome::Identical);
    statistics.register(OmfRepackOutcome::Mismatched);
    statistics.register(OmfRepackOutcome::Failed);

    assert_eq!(statistics.checked(), 3);
    assert_eq!(statistics.mismatched(), 1);
    assert_eq!(statistics.failed(), 1);
    assert_eq!(statistics.identical(), 1);
    assert!(!statistics.is_valid());
  }
}
