use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The asset roots over IPC, without owning them: the composition root manages the `AssetMountState`.
pub struct AssetsPlugin;

impl AssetsPlugin {
  /// The plugin, reading and committing through the mounted asset roots.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::assets::builder().build()
  }
}
