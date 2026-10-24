use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The running-work surface: what is going on, and asking it to stop.
pub struct JobsPlugin;

impl JobsPlugin {
  /// The plugin, listing, attaching to and cancelling the jobs the registry runs.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::jobs::builder().build()
  }
}
