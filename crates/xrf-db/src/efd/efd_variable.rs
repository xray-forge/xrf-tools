use serde::{Deserialize, Serialize};

/// One input of an evaluation function.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EfdVariable {
  /// Discrete values this input is bucketed into, which is what decides how big a pattern over it becomes.
  pub range: u32,
  /// The base function that supplies the value, by its own function type.
  pub kind: u32,
}

impl EfdVariable {
  /// Bytes one variable occupies across the two arrays that hold it.
  pub const SERIALIZED_SIZE: u64 = 4 + 4;
}
