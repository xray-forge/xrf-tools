use serde::{Deserialize, Serialize};

/// Optional human-facing documentation attached to an extern declaration.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExternDocumentation {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub returns: Option<String>,
}
