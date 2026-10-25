use serde::Serialize;

use crate::structure::{LtxStructureParent, LtxStructureScheme};

/// One section header, and what resolving the file it belongs to made of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureSection {
  /// One-based line the header was written on.
  pub line: u32,
  /// The name exactly as bracketed, padding included - `[ wpn_base ]` is not `[wpn_base]` and nothing inherits it.
  pub name: String,
  /// The header's operation prefix, spelled as the engine spells it: empty, `!`, `@` or `!!`.
  pub operation: String,
  pub parents: Vec<LtxStructureParent>,
  /// The scheme the resolved section is bound to, directly or through a parent.
  pub scheme: Option<LtxStructureScheme>,
}
