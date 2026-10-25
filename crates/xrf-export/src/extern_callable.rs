use serde::{Deserialize, Serialize};

use crate::extern_documentation::ExternDocumentation;
use crate::extern_parameter::ExternParameter;

/// A callable extern with an explicitly declared function contract.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExternCallable {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub doc: Option<ExternDocumentation>,
  pub params: Vec<ExternParameter>,
  pub returns: String,
  pub source: String,
}
