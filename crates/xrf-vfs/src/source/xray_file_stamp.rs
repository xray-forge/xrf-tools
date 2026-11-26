use std::fs::Metadata;
use std::time::{SystemTime, UNIX_EPOCH};

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

  /// The stamp folded into one number, which changes whenever the size or the modification time does: the time in
  /// milliseconds since the Unix epoch, zero where there is none, turned away from the size's low bits.
  pub fn get_fingerprint(&self) -> u64 {
    let modified_ms: u64 = self
      .modified
      .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
      .map_or(0, |since| u64::try_from(since.as_millis()).unwrap_or(u64::MAX));

    modified_ms.rotate_left(24) ^ self.size
  }
}
