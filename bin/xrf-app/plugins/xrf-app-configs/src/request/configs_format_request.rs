use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

/// What a config formatting run, or a check of one, was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsFormatRequest {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
}
