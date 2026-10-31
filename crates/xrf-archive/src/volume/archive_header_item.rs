/// One `name = value` line of a volume's `[header]`, as the engine loads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveHeaderItem {
  /// The name before the first `=`, trimmed and spelled as authored.
  pub key: String,
  /// The value after the first `=`, without its comment and without any whitespace outside quotes, whose quotes stay.
  /// Empty for a line with no `=` or nothing after it.
  pub value: String,
}
