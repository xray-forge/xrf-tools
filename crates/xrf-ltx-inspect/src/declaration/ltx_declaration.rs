use serde::Serialize;

use crate::declaration::LtxDeclarationStatement;

/// One statement that writes a section or a field, where it is written, and how.
///
/// Authored text, not a resolved value: a later patch can override or delete it. `LtxRootReader::read_sections`
/// answers which value survives.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxDeclaration {
  /// Engine identity of the config holding the statement.
  pub path: String,
  /// The mount that supplied that config, as the inventory names it.
  pub source: String,
  /// Whether a loose file backs that config.
  pub is_physical: bool,
  /// One-based line the statement was written on.
  pub line: u32,
  /// The header the statement sits under, or is; absent for a key written before the first header.
  pub section: Option<String>,
  /// That header's operation prefix: empty, `!`, `@` or `!!`.
  pub section_operation: String,
  /// Whether a prefix on the statement or its header makes it a patch, read from the text alone.
  ///
  /// Standard LTX refuses patches; under DLTX one edits what an earlier config declared.
  pub is_patch: bool,
  #[serde(flatten)]
  pub statement: LtxDeclarationStatement,
}
