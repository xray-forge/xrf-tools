use tauri::State;

use crate::core::window::WindowCloseApprovals;

/// Let the next close asked of a window, named by its label, go ahead: its page asked first and was answered.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "approve_window_close"))]
#[tauri::command(rename = "approve_window_close")]
pub fn system_approve_window_close(approvals: State<'_, WindowCloseApprovals>, window: String) {
  log::info!("Window '{window}' may close: its page agreed");

  approvals.approve(&window);
}
