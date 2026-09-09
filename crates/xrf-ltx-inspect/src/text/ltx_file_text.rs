use serde::Serialize;

/// One config's text, one entry per line, numbered the way every other record here anchors.
///
/// Lines rather than one string because every finding, every header and every include is addressed by line, and a
/// viewer that had to re-split the text would be a second place where "which line is this" is decided.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxFileText {
  /// Engine identity of the config these lines came from.
  pub path: String,
  /// Every line of the file, index `n` holding line `n + 1`.
  pub lines: Vec<String>,
  /// Whether the lines came back through the parser rather than as authored bytes.
  ///
  /// True is the normal answer and says one thing was lost: a line holding only whitespace comes back empty. It is
  /// recorded rather than hidden because an editor writing these lines back has to know they are not byte-identical.
  /// False means the file did not parse and the raw split was used instead.
  pub is_normalized: bool,
}
