use tauri::State;
use tauri::ipc::Channel;

use crate::assets::{AssetChangeEvent, AssetMountState};

/// Be told what every commit from now on changes: the project revision it leaves, and what it did to each open session.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "subscribe_changes"))]
#[tauri::command(rename = "subscribe_changes")]
pub fn assets_subscribe_changes(assets: State<'_, AssetMountState>, changes: Channel<AssetChangeEvent>) {
  log::info!("Subscribing to asset changes");

  assets.listen(changes);
}
