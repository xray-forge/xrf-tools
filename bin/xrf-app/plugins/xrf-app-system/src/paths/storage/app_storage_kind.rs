use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_app_core::cache::AppCache;
use xrf_app_core::directories::AppDirectories;

/// What a file the application keeps of its own is for, by where it lies in its data directory.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppStorageKind {
  /// What the renderer rebuilds when it is gone: the driver's compiled pipelines.
  RendererCache,
  /// Every other cache rebuilt from nothing.
  OtherCaches,
  /// This run's log and the last run's.
  Logs,
  /// The backend's preferences.
  Preferences,
  /// What a tool keeps for its user in a folder of its own (the level viewer's screenshots, bursts and views), which
  /// the application never removes.
  SavedFiles,
  /// Anything else kept there.
  Other,
}

impl AppStorageKind {
  /// Every kind, in the order a listing shows them.
  pub const ALL: [Self; 6] = [
    Self::RendererCache,
    Self::OtherCaches,
    Self::Logs,
    Self::Preferences,
    Self::SavedFiles,
    Self::Other,
  ];

  /// The kind of a file at `relative` within the application's data directory; none for the caches' own bookkeeping
  /// (`AppCache::is_own_file`), which holds nothing and would read as their last use.
  pub fn of(relative: &Path) -> Option<Self> {
    let parts: Vec<&str> = relative
      .components()
      .filter_map(|component| match component {
        Component::Normal(part) => part.to_str(),
        _ => None,
      })
      .collect();

    Some(match parts.as_slice() {
      [AppDirectories::CACHE_DIRECTORY, name] if AppCache::is_own_file(name) => return None,
      [AppDirectories::CACHE_DIRECTORY, AppDirectories::RENDERER_CACHE, _, ..] => Self::RendererCache,
      [AppDirectories::CACHE_DIRECTORY, _, ..] => Self::OtherCaches,
      [AppDirectories::LOGS_DIRECTORY, _, ..] => Self::Logs,
      [AppDirectories::PREFERENCES_FILE] => Self::Preferences,
      [_, _, ..] => Self::SavedFiles,
      _ => Self::Other,
    })
  }

  /// The folder within the application's data directory a kind may be cleared by removing, where it may be: only the
  /// renderer's cache, which it rebuilds as it starts.
  pub fn get_clearable_folder(self) -> Option<PathBuf> {
    match self {
      Self::RendererCache => Some(
        [AppDirectories::CACHE_DIRECTORY, AppDirectories::RENDERER_CACHE]
          .iter()
          .collect(),
      ),
      _ => None,
    }
  }
}
