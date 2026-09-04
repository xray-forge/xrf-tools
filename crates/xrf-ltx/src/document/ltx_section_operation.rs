/// What a section header's prefix asks for.
///
/// Only [`Self::Declare`] is standard LTX. The rest are DLTX, recorded so the document can carry a patch file without
/// pretending the prefix is part of the section name; see `xrf-agents/reports/20260902223702-dltx-compatibility-matrix.md`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LtxSectionOperation {
  /// `[section]` - declare it.
  #[default]
  Declare,
  /// `![section]` - override an existing one, dropped by the engine when it does not exist.
  Override,
  /// `@[section]` - override it, creating an empty base first when it does not exist.
  SafeOverride,
  /// `!![section]` - delete it after everything else resolves.
  Delete,
}

impl LtxSectionOperation {
  /// Whether this is a DLTX operation rather than a plain declaration.
  pub fn is_patch(&self) -> bool {
    !matches!(self, Self::Declare)
  }

  /// The prefix that spells it.
  pub fn as_prefix(&self) -> &'static str {
    match self {
      Self::Declare => "",
      Self::Override => "!",
      Self::SafeOverride => "@",
      Self::Delete => "!!",
    }
  }
}
