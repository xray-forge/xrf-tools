use xrf_report::Status;

use crate::commands::omf::repack::OmfRepackOutcome;

/// How the round trips of many omf files came out, counted.
#[derive(Default)]
pub struct OmfRepackStatistics {
  checked: u32,
  failed: u32,
  malformed: u32,
  mismatched: u32,
}

impl OmfRepackStatistics {
  /// Counts one file's round trip.
  pub fn register(&mut self, outcome: OmfRepackOutcome) {
    self.checked += 1;

    match outcome {
      OmfRepackOutcome::Identical => {}
      OmfRepackOutcome::Mismatched => self.mismatched += 1,
      OmfRepackOutcome::Malformed => self.malformed += 1,
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

  /// Count of files that were read but would not parse.
  pub fn malformed(&self) -> u32 {
    self.malformed
  }

  /// Count of files that could not be read or written.
  pub fn failed(&self) -> u32 {
    self.failed
  }

  /// Count of files judged defective: read, and either malformed or mismatched.
  pub fn judged_defective(&self) -> u32 {
    self.malformed + self.mismatched
  }

  /// Count of files that serialized back into identical bytes.
  pub fn identical(&self) -> u32 {
    self.checked - self.judged_defective() - self.failed
  }

  /// What the round trips came to: skipped where nothing was checked, an error where a file could not be read, failed
  /// where one was judged defective.
  pub fn status(&self) -> Status {
    if self.checked == 0 {
      Status::Skipped
    } else if self.failed > 0 {
      Status::Error
    } else {
      Status::from_is_valid(self.judged_defective() == 0)
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_report::Status;

  use crate::commands::omf::repack::{OmfRepackOutcome, OmfRepackStatistics};

  /// Nothing checked is not a pass.
  #[test]
  fn empty_statistics_are_skipped() {
    let statistics: OmfRepackStatistics = OmfRepackStatistics::default();

    assert_eq!(statistics.checked(), 0);
    assert_eq!(statistics.identical(), 0);
    assert_eq!(statistics.status(), Status::Skipped);
  }

  #[test]
  fn statistics_count_identical_outcomes() {
    let mut statistics: OmfRepackStatistics = OmfRepackStatistics::default();

    statistics.register(OmfRepackOutcome::Identical);
    statistics.register(OmfRepackOutcome::Identical);

    assert_eq!(statistics.checked(), 2);
    assert_eq!(statistics.identical(), 2);
    assert_eq!(statistics.status(), Status::Passed);
  }

  #[test]
  fn statistics_judge_malformed_and_mismatched_files_alike() {
    let mut statistics: OmfRepackStatistics = OmfRepackStatistics::default();

    statistics.register(OmfRepackOutcome::Identical);
    statistics.register(OmfRepackOutcome::Mismatched);
    statistics.register(OmfRepackOutcome::Malformed);

    assert_eq!(statistics.judged_defective(), 2);
    assert_eq!(statistics.identical(), 1);
    assert_eq!(statistics.status(), Status::Failed);
  }

  /// A file that could not be read outranks every verdict: the run did not judge it.
  #[test]
  fn statistics_answer_an_error_where_a_file_was_not_read() {
    let mut statistics: OmfRepackStatistics = OmfRepackStatistics::default();

    statistics.register(OmfRepackOutcome::Mismatched);
    statistics.register(OmfRepackOutcome::Failed);

    assert_eq!(statistics.failed(), 1);
    assert_eq!(statistics.identical(), 0);
    assert_eq!(statistics.status(), Status::Error);
  }
}
