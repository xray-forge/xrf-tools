use tauri::State;
use xrf_app_core::window::InvokingWindow;
use xrf_renderer::RenderHoldId;

use crate::RenderViewports;

/// Release a hold the calling window's page took; another window's hold is left alone. The GPU stops a few seconds
/// after the last hold and viewport go.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "release_hold")]
pub(crate) fn render_release_hold(window: InvokingWindow, viewports: State<'_, RenderViewports>, hold: RenderHoldId) {
  viewports.release_hold(window.get_label(), hold);
}
