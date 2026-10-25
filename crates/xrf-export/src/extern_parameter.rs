use serde::{Deserialize, Serialize};

/// One explicitly typed callable argument.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExternParameter {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub doc: Option<String>,
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub optional: Option<bool>,
  #[serde(rename = "type")]
  pub type_name: String,
}
