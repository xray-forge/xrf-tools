use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The `gamedata` plugin: whole-project gamedata checks.
pub struct GamedataPlugin;

impl GamedataPlugin {
  /// The plugin, which holds no state of its own.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::gamedata::builder().build()
  }
}
