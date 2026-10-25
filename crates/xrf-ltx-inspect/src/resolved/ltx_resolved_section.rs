use serde::Serialize;

use crate::resolved::LtxResolvedField;

/// One resolved section with its fields and where each of them came from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxResolvedSection {
  /// Engine identity of the entry point this section was resolved from, so a consumer can key a cache by it.
  pub entry: String,
  pub name: String,
  /// Parents the header declared, read back from the declaring config.
  pub parents: Vec<String>,
  /// Engine identity of the config whose header declared the section, where the dialect stamped one.
  pub origin: Option<String>,
  /// Fields in resolved order, which is written order with inherited ones folded in ahead of them.
  pub fields: Vec<LtxResolvedField>,
}
