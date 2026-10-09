use serde::{Deserialize, Serialize};

/// The modifier keys held as a drag's press went down.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInputModifiers {
  pub alt_key: bool,
  pub ctrl_key: bool,
  pub meta_key: bool,
  pub shift_key: bool,
}
