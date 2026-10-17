use serde::Serialize;

/// One payload that could not be read back.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVerifyFindingReport {
  message: String,
  name: String,
}

impl ArchiveVerifyFindingReport {
  pub fn new(name: &str, message: String) -> Self {
    Self {
      message,
      name: String::from(name),
    }
  }
}
