use serde::{Deserialize, Serialize};

/// What a mounted world is currently holding, and how well it has served reads.
///
/// Reported rather than inferred: retained bytes decide whether a session is healthy, and `refused` is the only signal
/// that a budget turned caching off, which would otherwise look like a mysterious slowdown.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayCacheStats {
  pub entries: usize,
  /// Source length of everything retained, used as the stand-in for parsed size.
  pub bytes: u64,
  pub hits: u64,
  pub misses: u64,
  /// Values parsed but not retained because the budget was already met.
  pub refused: u64,
}
