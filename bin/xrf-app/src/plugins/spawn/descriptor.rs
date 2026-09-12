use std::path::PathBuf;

use serde::Serialize;
use xrf_db::SpawnHeaderChunk;

use crate::core::session::SessionId;

/// One coherent opening, restored without reading the large chunks.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpawnSessionDescriptor {
  pub session_id: SessionId,
  pub path: PathBuf,
  pub header: SpawnHeaderChunk,
}
