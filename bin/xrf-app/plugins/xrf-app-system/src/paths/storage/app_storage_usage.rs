use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::paths::storage::app_storage_kind::AppStorageKind;

/// How much of one kind the application keeps.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStorageArea {
  pub kind: AppStorageKind,
  pub bytes: u64,
  pub files: u64,
}

/// What the application keeps in its own data directory, measured file by file: where that is, and how much of each
/// kind, every kind listed whether it holds anything or not.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStorageUsage {
  pub root: String,
  pub bytes: u64,
  pub areas: Vec<AppStorageArea>,
}

impl AppStorageUsage {
  /// Measures every file under `root`, the application's data directory; one that cannot be read counts nothing.
  pub fn measure(root: &Path) -> Self {
    let mut areas: Vec<AppStorageArea> = AppStorageKind::ALL
      .map(|kind| AppStorageArea {
        kind,
        bytes: 0,
        files: 0,
      })
      .to_vec();
    let mut pending: Vec<std::path::PathBuf> = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop() {
      let Ok(entries) = fs::read_dir(&directory) else {
        continue;
      };

      for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
          continue;
        };

        if metadata.is_dir() {
          pending.push(entry.path());

          continue;
        }

        let path = entry.path();
        let kind: AppStorageKind = AppStorageKind::of(path.strip_prefix(root).unwrap_or(&path));

        if let Some(area) = areas.iter_mut().find(|area| area.kind == kind) {
          area.bytes += metadata.len();
          area.files += 1;
        }
      }
    }

    Self {
      root: root.to_string_lossy().into_owned(),
      bytes: areas.iter().map(|area| area.bytes).sum(),
      areas,
    }
  }
}
