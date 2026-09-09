use serde::Serialize;

/// One finding, already placed at the file and line a person has to open.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxAnchoredFinding {
  pub kind: LtxFindingKind,
  /// Engine identity of the entry point whose resolution the finding was produced under.
  pub entry: String,
  /// Engine identity of the config to open, where one is known.
  ///
  /// `None` means nothing recorded which config declares the section, which a patch dialect answers for a section
  /// created by an override of something nothing declares.
  pub file: Option<String>,
  /// One-based line in `file`, where the anchor reached one.
  pub line: Option<u32>,
  pub section: Option<String>,
  pub field: Option<String>,
  pub message: String,
  /// What the engine does with the same input, where the dialect said so.
  pub engine_behaviour: Option<String>,
}

/// What kind of thing went wrong, which is also what decides how it was anchored.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LtxFindingKind {
  /// The file would not parse. Carries its own line, from the parser.
  Parse,
  /// A section broke the scheme it is bound to, or is bound to one nothing declares.
  Scheme,
  /// The dialect had something to say about the root that is not a failure.
  Dialect,
  /// An `#include` reached no file.
  Include,
}
