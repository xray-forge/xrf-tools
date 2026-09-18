use tauri::State;
use xrf_vfs::{XrayDirectoryListing, XrayProbe, XrayRoots};

use crate::core::assets::AssetMountState;
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{GEOMETRY_FILE, LEVEL_FILE, LEVELS_DIRECTORY, LevelEntry};

/// Every compiled level the mounted roots hold, loose or archived alike.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_levels"))]
#[tauri::command(rename = "list_levels")]
pub async fn levels_list_levels(roots: XrayRoots, assets: State<'_, AssetMountState>) -> TauriResult<Vec<LevelEntry>> {
  let entries: Vec<LevelEntry> = assets.with_probe(&roots, list_levels)??;

  log::info!("Listed {} levels in the mounted roots", entries.len());

  Ok(entries)
}

/// Walks `levels` and keeps the names that are levels.
fn list_levels(probe: &XrayProbe) -> TauriResult<Vec<LevelEntry>> {
  let listing: XrayDirectoryListing = probe
    .list_children(LEVELS_DIRECTORY)
    .map_err(|error| format!("Failed to list levels: {error}"))?;

  let mut entries: Vec<LevelEntry> = Vec::new();

  for name in listing.directories {
    let logical_path: String = format!("{LEVELS_DIRECTORY}\\{name}");

    if !is_present(probe, &logical_path, LEVEL_FILE) {
      continue;
    }

    entries.push(LevelEntry {
      // A level shipping no render geometry draws nothing, which is worth saying in the list rather than on open.
      has_geometry: is_present(probe, &logical_path, GEOMETRY_FILE),
      logical_path,
      name,
    });
  }

  Ok(entries)
}

/// Whether one of a level's files is there, treating an unreadable path as absent rather than failing the listing.
fn is_present(probe: &XrayProbe, logical_path: &str, file: &str) -> bool {
  probe
    .find(&format!("{logical_path}\\{file}"))
    .is_ok_and(|resolution| resolution.get_asset().is_some())
}
