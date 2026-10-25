use serde::{Deserialize, Serialize};

/// The counts at one moment, detached from the project that produced them.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxReadCountersSnapshot {
  /// Configs read out of the mounted sources, counting a file once per read rather than once per distinct path.
  pub reads: u64,
  /// Source bytes those reads returned.
  pub bytes_read: u64,
  /// Full parses of a config into sections.
  pub parses: u64,
  /// Include-only parses, which project assembly performs over every config before any content is parsed.
  pub include_scans: u64,
  /// Roots resolved with includes merged and inheritance flattened.
  pub resolutions: u64,
}
