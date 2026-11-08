/// A value a later line of the same key replaced on reading, under [`crate::IniRepeatedKey::LastValueKept`]: it is not
/// in the file read, and a write leaves it out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IniOverride {
  pub section: String,
  pub key: String,
  /// The value dropped.
  pub value: String,
  /// The line it stood on, one-based.
  pub line: usize,
}
