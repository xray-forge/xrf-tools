use tauri::State;
use xrf_renderer::RenderHoldId;

use crate::RenderViewports;

/// Let a hold on the renderer go; the GPU goes a few seconds after the last hold and viewport.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "release_hold")]
pub(crate) fn render_release_hold(viewports: State<'_, RenderViewports>, hold: RenderHoldId) {
  viewports.release_hold(hold);
}
