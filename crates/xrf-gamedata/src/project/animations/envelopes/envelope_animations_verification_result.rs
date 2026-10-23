use std::marker::PhantomData;

use crate::project::animations::envelopes::envelope_animation::EnvelopeAnimation;
use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

/// Aggregated outcome of reading, writing back and evaluating every file of one envelope animation format.
pub(crate) struct GamedataEnvelopeAnimationsVerificationResult<F> {
  pub(crate) checked_files_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_files_count: u32,
  // Names the format without holding one, as the verifier does.
  pub(crate) format: PhantomData<fn() -> F>,
}

impl<F> Default for GamedataEnvelopeAnimationsVerificationResult<F> {
  fn default() -> Self {
    Self {
      checked_files_count: 0,
      findings: Vec::new(),
      invalid_files_count: 0,
      format: PhantomData,
    }
  }
}

impl<F: EnvelopeAnimation> GamedataCheckResult for GamedataEnvelopeAnimationsVerificationResult<F> {
  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_files_count == 0)
  }

  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} {} valid",
      self.checked_files_count - self.invalid_files_count,
      self.checked_files_count,
      F::PLURAL_NAME
    )
  }

  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}

#[cfg(test)]
mod tests {
  use xrf_anm::AnmFile;
  use xrf_ppe::PpeFile;

  use super::GamedataEnvelopeAnimationsVerificationResult;
  use crate::{GamedataCheckResult, GamedataVerificationStatus};

  #[test]
  fn names_the_format_it_counts() {
    let motions: GamedataEnvelopeAnimationsVerificationResult<AnmFile> = GamedataEnvelopeAnimationsVerificationResult {
      checked_files_count: 3,
      invalid_files_count: 1,
      ..Default::default()
    };

    assert_eq!(motions.get_failure_message(), "2/3 object motions valid");
    assert_eq!(motions.get_status(), GamedataVerificationStatus::Failed);
    assert_eq!(
      GamedataEnvelopeAnimationsVerificationResult::<PpeFile>::default().get_failure_message(),
      "0/0 post-process effects valid"
    );
  }
}
