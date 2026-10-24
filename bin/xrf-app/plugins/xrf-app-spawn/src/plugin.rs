use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::session::SessionScopes;

use crate::SpawnFiles;

/// The `spawn` plugin: spawns opened in the editors, read chunk by chunk and saved, and conversions run as jobs.
pub struct SpawnPlugin;

impl SpawnPlugin {
  /// The plugin, its spawn files managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::spawn::builder()
      .setup(|application, _| {
        application.manage(SpawnFiles::new("spawn file", &application.state::<SessionScopes>()));

        Ok(())
      })
      .build()
  }
}
