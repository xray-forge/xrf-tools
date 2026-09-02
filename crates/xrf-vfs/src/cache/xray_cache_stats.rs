use serde::{Deserialize, Serialize};

/// What a mounted world is currently holding, and how well it has served reads.
///
/// Reported rather than inferred: retained bytes decide whether a session is healthy, and `refused` is the only signal
/// that a budget turned caching off, which would otherwise look like a mysterious slowdown.
///
/// These numbers are published, so they describe the inputs rather than the run. Two sweeps over one tree report the
/// same figures however many threads they used, which is what makes two reports comparable. That holds while the policy
/// carries no byte budget; see [`crate::XrayCachePolicy::with_budget`].
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
