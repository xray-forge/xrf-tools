use serde::{Deserialize, Serialize};

/// How long an application's caches keep what goes unused, and how much they keep at most.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCachePolicy {
  /// Days an entry may go unread before it is removed.
  pub unused_days: u32,
  /// Bytes every cache together may hold, past which the least recently used entries are removed first.
  pub size_limit: u64,
}

/// A week unused, as Unreal's local derived data cache keeps eight days; five gibibytes, ccache's default.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const DEFAULT_APP_CACHE_POLICY: AppCachePolicy = AppCachePolicy {
  unused_days: 7,
  size_limit: 5 << 30,
};

impl Default for AppCachePolicy {
  fn default() -> Self {
    DEFAULT_APP_CACHE_POLICY
  }
}
