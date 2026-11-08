use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::window::InvokingWindow;
use xrf_renderer::RenderHoldId;

use crate::RenderViewports;

/// Keep the renderer's GPU started for the calling window's page, starting it where it is not, so a viewport attached
/// later draws at once: for a tool whose viewports come and go. Held until released, the page goes or the window closes.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "hold")]
pub(crate) fn render_hold(viewports: State<'_, RenderViewports>, window: InvokingWindow) -> TauriResult<RenderHoldId> {
  viewports.hold(window.get_label())
}
