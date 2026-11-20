/// What a read does with a key set a second time in one section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IniRepeatedKey {
  /// The read fails.
  Refused,
  /// The later value replaces the earlier, as `CInifile::Load` does (`insert_item`); each value dropped this way is
  /// named in [`crate::IniFile::overridden`].
  LastValueKept,
}
