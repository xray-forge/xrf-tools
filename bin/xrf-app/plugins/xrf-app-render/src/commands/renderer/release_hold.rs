use tauri::State;
use xrf_app_core::window::InvokingWindow;
use xrf_renderer::RenderHoldId;

use crate::RenderViewports;

/// Let a hold the asking window's page took on the renderer go; the GPU goes a few seconds after the last hold and
/// viewport. Another window's hold is left alone.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "release_hold")]
pub(crate) fn render_release_hold(window: InvokingWindow, viewports: State<'_, RenderViewports>, hold: RenderHoldId) {
  viewports.release_hold(window.get_label(), hold);
}
