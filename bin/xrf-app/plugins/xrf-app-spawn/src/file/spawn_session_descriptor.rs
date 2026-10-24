use std::path::PathBuf;

use serde::Serialize;
use xrf_spawn::SpawnHeaderChunk;

/// What an opened spawn is, without its large chunks: where it was read from and its header, which an open and a
/// restore answer under the opening's identity.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpawnSessionDescriptor {
  pub path: PathBuf,
  pub header: SpawnHeaderChunk,
}
