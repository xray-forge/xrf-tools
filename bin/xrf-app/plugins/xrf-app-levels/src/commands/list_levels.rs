use std::time::Instant;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::types::TauriResult;
use xrf_vfs::{XrayDirectoryListing, XrayLogicalPath, XrayRoots, XraySearch};

use crate::level_report::LevelReport;
use crate::state::{GEOMETRY_FILE, LEVEL_FILE, LEVELS_DIRECTORY, LevelEntry};

/// Every compiled level the mounted roots hold, loose or archived alike.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_levels"))]
#[tauri::command(rename = "list_levels")]
pub async fn levels_list_levels(roots: XrayRoots, assets: State<'_, AssetMountState>) -> TauriResult<Vec<LevelEntry>> {
  let started: Instant = Instant::now();
  let entries: Vec<LevelEntry> = list_levels(&assets.search_fresh(&roots)?)?;

  LevelReport::listed_levels(&entries, started);

  Ok(entries)
}

/// Walks `levels` and keeps the names that are levels.
fn list_levels(search: &XraySearch) -> TauriResult<Vec<LevelEntry>> {
  let levels: XrayLogicalPath =
    XrayLogicalPath::new(LEVELS_DIRECTORY).map_err(|error| format!("Failed to list levels: {error}"))?;
  let listing: XrayDirectoryListing = search
    .list_children(levels.as_str())
    .map_err(|error| format!("Failed to list levels: {error}"))?;

  let mut entries: Vec<LevelEntry> = Vec::new();

  for name in listing.directories {
    let Ok(logical_path) = levels.join(&name) else {
      continue;
    };

    if !is_present(search, &logical_path, LEVEL_FILE) {
      continue;
    }

    entries.push(LevelEntry {
      // A level shipping no render geometry draws nothing, which is worth saying in the list rather than on open.
      has_geometry: is_present(search, &logical_path, GEOMETRY_FILE),
      logical_path: logical_path.as_str().to_owned(),
      name,
    });
  }

  Ok(entries)
}

/// Whether one of a level's files is there, treating an unreadable path as absent rather than failing the listing.
fn is_present(search: &XraySearch, logical_path: &XrayLogicalPath, file: &str) -> bool {
  logical_path
    .join(file)
    .and_then(|path| search.find(path.as_str()))
    .is_ok_and(|found| found.is_some())
}
