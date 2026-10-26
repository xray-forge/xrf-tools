use std::path::PathBuf;

use crate::XraySourceKind;

/// One source to mount before it is opened or indexed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct XrayPlannedMount {
  /// Where the source lives.
  pub path: PathBuf,
  /// Logical base the source mounts at, empty for a root.
  pub base: String,
  /// Source implementation to construct for this path.
  pub kind: XraySourceKind,
  /// Caller-supplied diagnostic label, such as an `fsgame.ltx` alias.
  pub origin: String,
  /// Logical prefixes this source omits when indexed.
  pub ignored: Vec<String>,
}
