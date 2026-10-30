use serde::Serialize;

/// What a tidy of the application's caches did: whether it ran, what it removed and what it kept.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCacheTidy {
  /// Whether it ran: not when another instance holds the caches, nor when one ran within the day and none was forced.
  pub is_done: bool,
  pub removed_entries: u64,
  pub removed_bytes: u64,
  pub kept_entries: u64,
  pub kept_bytes: u64,
}
