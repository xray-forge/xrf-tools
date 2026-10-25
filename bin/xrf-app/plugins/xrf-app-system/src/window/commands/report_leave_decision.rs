use crate::window::LeaveDecision;

/// Write what the page decided about leaving or closing to the log, beside what the backend did about it.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "report_leave_decision")]
pub fn system_report_leave_decision(decision: LeaveDecision, unsaved: u32) {
  log::info!("Page decided: {} ({unsaved} unsaved)", decision.describe());
}
