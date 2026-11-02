use serde::Serialize;

use crate::resolved::LtxResolvedField;
use crate::scheme::LtxSchemeFieldDeclaration;

/// One row of a scheme report: what the scheme asks for, and what the section answers.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxSchemeFieldReport {
  pub name: String,
  /// What the scheme declares about it, absent for a field the section holds and no declaration covers.
  pub declared: Option<LtxSchemeFieldDeclaration>,
  /// What the section resolves to, absent for a declared field the section does not hold.
  pub resolved: Option<LtxResolvedField>,
}
