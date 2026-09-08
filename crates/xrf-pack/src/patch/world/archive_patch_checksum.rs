/// One side's checksum for an entry, and what learning it cost.
///
/// A value rather than a `(u32, bool)` pair, because the bool answers a question a caller cannot guess from the
/// number: whether the comparison had to decompress or read a payload to obtain it. That is the whole difference
/// between the cheap and the expensive shape of the same run, and it is what the result reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArchivePatchChecksum {
  /// Read from an archive's name table, where the packer wrote it and the engine verifies it. Free.
  Recorded(u32),
  /// Computed here, because the source holds no checksum of its own. Cost one full read of the payload.
  Hashed(u32),
}

impl ArchivePatchChecksum {
  /// The checksum itself, however it was obtained.
  pub(crate) const fn get_value(self) -> u32 {
    match self {
      Self::Recorded(crc) | Self::Hashed(crc) => crc,
    }
  }

  /// Whether obtaining it cost a payload read.
  pub(crate) const fn is_hashed(self) -> bool {
    matches!(self, Self::Hashed(_))
  }
}
