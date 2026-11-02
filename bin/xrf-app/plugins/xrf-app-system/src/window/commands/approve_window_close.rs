use tauri::State;
use xrf_app_core::window::{InvokingWindow, WindowCloseApprovals};

/// Let the next close asked of the calling window go ahead: its page asked first and was answered.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "approve_window_close")]
pub fn system_approve_window_close(approvals: State<'_, WindowCloseApprovals>, window: InvokingWindow) {
  let label: &str = window.get_label();

  log::info!("Window '{label}' may close: its page agreed");

  approvals.approve(label);
}
