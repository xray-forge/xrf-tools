use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
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
  /// Anything else kept there.
  Other,
}

impl AppStorageKind {
  /// Every kind, in the order a listing shows them.
  pub const ALL: [Self; 5] = [
    Self::RendererCache,
    Self::OtherCaches,
    Self::Logs,
    Self::Preferences,
    Self::Other,
  ];

  /// The kind of a file at `relative` within the application's data directory.
  pub fn of(relative: &Path) -> Self {
    let parts: Vec<&str> = relative
      .components()
      .filter_map(|component| match component {
        Component::Normal(part) => part.to_str(),
        _ => None,
      })
      .collect();

    match parts.as_slice() {
      [AppDirectories::CACHE_DIRECTORY, AppDirectories::RENDERER_CACHE, _, ..] => Self::RendererCache,
      [AppDirectories::CACHE_DIRECTORY, _, ..] => Self::OtherCaches,
      [AppDirectories::LOGS_DIRECTORY, _, ..] => Self::Logs,
      [AppDirectories::PREFERENCES_FILE] => Self::Preferences,
      _ => Self::Other,
    }
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
