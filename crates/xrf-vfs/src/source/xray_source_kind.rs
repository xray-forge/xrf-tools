use serde::Serialize;

/// The storage kind backing a mount.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XraySourceKind {
  /// Loose files under a directory.
  Directory,
  /// Entries inside a set of `.db` archive volumes.
  Archive,
}

impl XraySourceKind {
  /// The kind as it is serialized, for a line that names it the same way a report does.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Directory => "directory",
      Self::Archive => "archive",
    }
  }
}
