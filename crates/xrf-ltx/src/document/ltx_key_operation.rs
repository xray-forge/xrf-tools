/// What a field line's prefix asks for.
///
/// Only [`Self::Set`] is standard LTX; the rest are DLTX. Recorded on the key rather than as separate statement kinds
/// because the engine itself routes on the first character of the key name.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LtxKeyOperation {
  /// `key = value` - set it.
  #[default]
  Set,
  /// `!key` - delete it, discarding any value written after it.
  Delete,
  /// `>key = a, b` - append to its comma list.
  ListAppend,
  /// `<key = a, b` - remove from its comma list.
  ListRemove,
}

impl LtxKeyOperation {
  /// Whether this is a DLTX operation rather than a plain assignment.
  pub fn is_patch(&self) -> bool {
    !matches!(self, Self::Set)
  }

  /// The prefix that spells it.
  pub fn as_prefix(&self) -> &'static str {
    match self {
      Self::Set => "",
      Self::Delete => "!",
      Self::ListAppend => ">",
      Self::ListRemove => "<",
    }
  }
}
