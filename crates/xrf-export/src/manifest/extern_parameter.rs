use serde::{Deserialize, Serialize};

/// One explicitly typed callable argument.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExternParameter {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub doc: Option<String>,
  pub name: String,
  /// `Some(true)` for an optional parameter and absent otherwise, which keeps the tracked artifact's shape.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub optional: Option<bool>,
  #[serde(rename = "type")]
  pub type_name: String,
}

impl ExternParameter {
  /// Whether a caller may leave the parameter out.
  pub fn is_optional(&self) -> bool {
    self.optional.is_some_and(|optional| optional)
  }

  /// The parameter as TypeScript writes it in a signature: `name?: Type`.
  pub fn get_contract(&self) -> String {
    format!(
      "{}{}: {}",
      self.name,
      if self.is_optional() { "?" } else { "" },
      self.type_name
    )
  }
}
