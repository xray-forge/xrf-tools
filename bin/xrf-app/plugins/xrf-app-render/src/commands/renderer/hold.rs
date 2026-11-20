use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::window::InvokingWindow;
use xrf_renderer::RenderHoldId;

use crate::RenderViewports;

/// Start the renderer's GPU and keep it for the calling window's page, so a tool whose viewports come and go draws at
/// once. Held until released, the page reloads or the window closes.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "hold")]
pub(crate) fn render_hold(viewports: State<'_, RenderViewports>, window: InvokingWindow) -> TauriResult<RenderHoldId> {
  viewports.hold(window.get_label())
}
