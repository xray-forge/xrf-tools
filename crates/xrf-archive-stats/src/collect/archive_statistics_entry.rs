use xrf_archive::ArchiveFileDescriptor;

/// One entry a breakdown measures: what it is called, and what it weighs.
pub trait ArchiveStatisticsEntry {
  /// Engine identity, backslash separated, as every surface addresses this entry by.
  fn get_name(&self) -> &str;

  /// Payload bytes once unpacked.
  fn get_size_real(&self) -> u64;

  /// Payload bytes as stored, for a subject whose format records one.
  fn get_size_compressed(&self) -> Option<u64> {
    None
  }

  /// Whether this entry names a directory rather than a file holding bytes.
  fn is_directory(&self) -> bool {
    false
  }
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
