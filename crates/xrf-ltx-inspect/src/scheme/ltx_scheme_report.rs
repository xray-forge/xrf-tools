use serde::Serialize;

use crate::resolved::LtxResolvedField;

/// What judges one resolved section, and how the section measures against it.
///
/// The scheme is read off the resolution rather than off the file, so a section that inherits its binding is judged by
/// the same rule the verifier judges it by - which is the whole reason a binding is worth showing: nothing in the text
/// of `[wpn_child]:wpn_base` says it is a weapon.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxSectionSchemeReport {
  /// Engine identity of the entry point whose resolution this was read from.
  pub entry: String,
  pub section: String,
  /// The `$scheme` the resolved section carries, absent when it carries none.
  pub scheme: Option<String>,
  /// Whether a scheme file declares that name. False is itself a finding, and the verifier reports it as one.
  pub is_declared: bool,
  /// Whether the declaration refuses fields it does not name and demands the ones it does not mark optional.
  pub is_strict: bool,
  /// The section the binding is written in, absent when this section writes it itself.
  pub inherited_from: Option<String>,
  /// Every field the scheme declares and every field the section holds, merged.
  pub fields: Vec<LtxSchemeFieldReport>,
}

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

/// What a scheme declares about one field.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxSchemeFieldDeclaration {
  /// The type as the scheme spells it - `u32`, `enum:pistol,rifle`, `condlist`.
  pub data_type: String,
  pub is_array: bool,
  pub is_optional: bool,
  /// Whether this is the scheme's catch-all `*` rather than a declaration naming the field.
  pub is_any: bool,
}
