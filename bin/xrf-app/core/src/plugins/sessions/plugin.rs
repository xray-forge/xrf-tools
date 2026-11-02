use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The scopes surface: binding an opening to the tool that asked for it, and releasing what a tool or a page left.
pub struct SessionsPlugin;

impl SessionsPlugin {
  /// The plugin, binding and releasing the scopes sessions are held by.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::sessions::builder().build()
  }
}
