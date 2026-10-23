use serde::{Deserialize, Serialize};

/// What a VFS snapshot is currently holding, and how well it has served reads.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayCacheStats {
  pub entries: usize,
  /// Source length of everything retained, used as the stand-in for parsed size.
  pub bytes: u64,
  /// Reads served from what was already retained, so no parse was needed.
  pub hits: u64,
  /// Reads that performed the parse. With `hits`, the number of parse requests the run made.
  pub misses: u64,
  /// Values parsed but not retained because the budget was already met.
  pub refused: u64,
}
