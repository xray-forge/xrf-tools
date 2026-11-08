use serde::Serialize;

use crate::declaration::LtxDeclarationStatement;

/// One statement that writes a section or a field, where it is written, and how.
///
/// This is authored text, not a resolved value: a field found here can be overridden by a later patch, deleted, or
/// never reached at all. Which value survives is what `LtxRootReader::read_sections` answers.
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
  /// That header's operation prefix, spelled as the engine spells it: empty, `!`, `@` or `!!`.
  pub section_operation: String,
  /// Whether the statement is written as a patch, by a prefix on itself or on its header.
  ///
  /// Read from the text alone. Standard LTX refuses every such statement; under DLTX it edits whatever an earlier
  /// config declared.
  pub is_patch: bool,
  #[serde(flatten)]
  pub statement: LtxDeclarationStatement,
}
