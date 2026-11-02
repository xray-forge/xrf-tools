use serde::Serialize;

use crate::resolved::LtxResolvedFieldOrigin;

/// One resolved field: what it says, and why it says that.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxResolvedField {
  pub key: String,
  pub value: String,
  pub origin: LtxResolvedFieldOrigin,
}
