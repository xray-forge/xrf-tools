use serde::Serialize;

/// What a volume set's payloads compress to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveCompression {
  /// Bytes the payloads occupy as stored.
  pub size_compressed: u64,
  /// Bytes they occupy once unpacked.
  pub size_real: u64,
  /// Entries whose stored size equals their unpacked size, which is how the format says "stored uncompressed".
  pub stored_uncompressed: u64,
}
