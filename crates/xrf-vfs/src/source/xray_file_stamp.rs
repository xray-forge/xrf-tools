use std::fs::Metadata;
use std::time::SystemTime;

/// How a loose file looked on disk when it was read: its size and modification time, compared later to tell whether
/// it changed behind every reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XrayFileStamp {
  size: u64,
  /// `None` where the filesystem keeps no modification time, which leaves the size as the whole comparison.
  modified: Option<SystemTime>,
}

impl XrayFileStamp {
  /// The stamp a file's metadata gives it.
  pub fn of(metadata: &Metadata) -> Self {
    Self {
      size: metadata.len(),
      modified: metadata.modified().ok(),
    }
  }
}
