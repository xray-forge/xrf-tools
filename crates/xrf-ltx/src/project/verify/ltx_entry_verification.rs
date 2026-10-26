use crate::finding::LtxFinding;

/// What verifying one resolved entry point found, and how much of it was judged.
///
/// Separate from [`crate::LtxProjectVerifyResult`], which is a whole run: this has no outcome, no duration and no file
/// count, because one entry point neither starts nor stops a run. A sweep folds these together and adds those.
#[derive(Debug, Default)]
pub struct LtxEntryVerification {
  pub findings: Vec<LtxFinding>,
  pub checked_fields: usize,
  pub checked_sections: usize,
  pub invalid_sections: usize,
  pub skipped_sections: usize,
  pub total_sections: usize,
  pub valid_sections: usize,
}
