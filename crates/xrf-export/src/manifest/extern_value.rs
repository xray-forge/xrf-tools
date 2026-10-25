use serde::{Deserialize, Serialize};

use crate::manifest::extern_documentation::ExternDocumentation;

/// A non-callable extern with an explicitly asserted value type.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExternValue {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub doc: Option<ExternDocumentation>,
  pub source: String,
  #[serde(rename = "type")]
  pub type_name: String,
}
