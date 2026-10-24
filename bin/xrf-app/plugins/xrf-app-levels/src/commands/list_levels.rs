use std::time::Instant;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_vfs::XrayRoots;

use crate::level::LevelEntry;
use crate::level_report::LevelReport;

/// Every compiled level the mounted roots hold, loose or archived alike, listed on the pool from a fresh search.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list_levels"))]
#[tauri::command(rename = "list_levels")]
pub async fn levels_list_levels(
  roots: XrayRoots,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<Vec<LevelEntry>> {
  let assets: AssetMountState = AssetMountState::clone(&assets);

  execution
    .run_blocking("Listing levels", move || {
      let started: Instant = Instant::now();
      let entries: Vec<LevelEntry> = LevelEntry::list(&assets.search_fresh(&roots)?)?;

      LevelReport::listed_levels(&entries, started);

      Ok(entries)
    })
    .await?
}
