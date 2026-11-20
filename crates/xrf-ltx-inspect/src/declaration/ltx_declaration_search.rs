use serde::Serialize;

use crate::declaration::{LtxDeclaration, LtxDeclarationQuery, LtxUnreadableConfig};

/// What one declaration search found across a project.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxDeclarationSearch {
  /// The query, echoed back.
  pub query: LtxDeclarationQuery,
  /// Every matching statement, in the inventory's config order and then in document order.
  pub declarations: Vec<LtxDeclaration>,
  /// Configs that would not parse, and so declare nothing this search could see.
  pub unreadable: Vec<LtxUnreadableConfig>,
}
