use serde::Serialize;

/// What kind of thing went wrong.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LtxFindingKind {
  /// The file would not parse. Carries its own line, from the parser.
  Parse,
  /// A section broke the scheme it is bound to, or is bound to one nothing declares.
  Scheme,
  /// The dialect had something to say about the root: a statement the engine loads and then silently ignores.
  Dialect,
  /// An `#include` reached no file.
  Include,
  /// A config declares keys above its first section header, where no engine loads them. Not raised for a config
  /// that declares no section at all: that is a list, read as data rather than loaded as an ini.
  RootKeys,
  /// An entry point that could not be resolved, so nothing reached through it was judged.
  Unreadable,
}

impl LtxFindingKind {
  /// How a line of plain text introduces a finding of this kind.
  pub const fn get_label(self) -> &'static str {
    match self {
      Self::Parse => "Ltx parse error",
      Self::Scheme => "Ltx scheme error",
      Self::Dialect => "Ltx dialect finding",
      Self::Include => "Ltx include error",
      Self::RootKeys => "Ltx root keys",
      Self::Unreadable => "Ltx unreadable entry",
    }
  }
}
