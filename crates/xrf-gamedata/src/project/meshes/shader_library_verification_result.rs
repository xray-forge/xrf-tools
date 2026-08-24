use std::sync::Arc;

use xrf_db::ShaderLibraryFile;

use crate::{Finding, GamedataCheckResult, GamedataVerificationStatus};

pub(crate) struct GamedataShaderLibraryVerificationResult {
  pub(crate) blender_count: usize,
  pub(crate) checked_count: u32,
  pub(crate) findings: Vec<Finding>,
  pub(crate) invalid_count: u32,
  library: Option<Arc<ShaderLibraryFile>>,
}

impl GamedataShaderLibraryVerificationResult {
  pub(crate) fn passed(library: Arc<ShaderLibraryFile>) -> Self {
    Self {
      blender_count: library.blenders_count(),
      checked_count: 1,
      findings: Vec::new(),
      invalid_count: 0,
      library: Some(library),
    }
  }

  pub(crate) fn failed(finding: Finding) -> Self {
    Self {
      blender_count: 0,
      checked_count: 1,
      findings: vec![finding],
      invalid_count: 1,
      library: None,
    }
  }

  pub(crate) fn library(&self) -> Option<&ShaderLibraryFile> {
    self.library.as_deref()
  }
}

impl GamedataCheckResult for GamedataShaderLibraryVerificationResult {
  fn get_status(&self) -> GamedataVerificationStatus {
    GamedataVerificationStatus::from_is_valid(self.invalid_count == 0)
  }
  fn get_failure_message(&self) -> String {
    format!(
      "{}/{} shader libraries valid, {} blender definitions",
      self.checked_count - self.invalid_count,
      self.checked_count,
      self.blender_count
    )
  }
  fn get_findings(&self) -> &[Finding] {
    &self.findings
  }
}
