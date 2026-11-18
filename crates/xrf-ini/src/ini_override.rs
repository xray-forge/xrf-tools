/// A value a later line of the same key replaced on reading, under [`crate::IniRepeatedKey::LastValueKept`].
///
/// A write leaves it out, as the engine does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IniOverride {
  pub section: String,
  pub key: String,
  /// The value dropped.
  pub value: String,
  /// The line it stood on, one-based.
  pub line: usize,
}
