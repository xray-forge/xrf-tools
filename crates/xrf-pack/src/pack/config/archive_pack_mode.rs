use serde::{Deserialize, Serialize};

/// How file payloads are stored in the archive.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchivePackMode {
  /// Compress what the engine expects to be compressed and store the rest.
  #[default]
  Compress,
  /// Store everything, the `-store` flag of xrCompress.
  Store,
}
