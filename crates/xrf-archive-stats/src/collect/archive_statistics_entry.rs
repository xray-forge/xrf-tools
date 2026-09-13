use xrf_archive::ArchiveFileDescriptor;
use xrf_vfs::XrayAssetContainer;

/// One entry a breakdown measures: what it is called, and what it weighs.
pub trait ArchiveStatisticsEntry {
  /// Engine identity, backslash separated, as every surface addresses this entry by.
  fn get_name(&self) -> &str;

  /// Payload bytes once unpacked.
  fn get_size_real(&self) -> u64;

  /// Payload bytes as stored, for a subject whose format records one.
  ///
  /// `None` is the honest answer for a mounted world rather than a repeat of the unpacked size: a loose file has no
  /// stored size, and reporting one would make every compression figure a fiction.
  fn get_size_compressed(&self) -> Option<u64> {
    None
  }

  /// Whether this entry names a directory rather than a file holding bytes.
  ///
  /// Volumes record the directories they contain so an unpacker can recreate them. Nothing that measures payloads may
  /// count them.
  fn is_directory(&self) -> bool {
    false
  }
}

/// What a mounted world adds: where the winning copy sits, and the sized copies it hides.
///
/// Separate from [`ArchiveStatisticsEntry`] because a volume set cannot answer either question, and a trait it could
/// only implement by inventing answers is worse than one it does not implement.
pub trait ArchiveWorldStatisticsEntry: ArchiveStatisticsEntry {
  /// Where the winning copy physically sits.
  fn get_container(&self) -> &XrayAssetContainer;

  /// Copies of this engine path no lookup reaches, each with its own unpacked size.
  fn list_shadowed(&self) -> impl Iterator<Item = (&XrayAssetContainer, u64)>;
}

impl ArchiveStatisticsEntry for ArchiveFileDescriptor {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn get_size_real(&self) -> u64 {
    u64::from(self.size_real)
  }

  fn get_size_compressed(&self) -> Option<u64> {
    Some(u64::from(self.size_compressed))
  }

  fn is_directory(&self) -> bool {
    self.is_directory
  }
}
