use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::session::SessionScopes;

use crate::ExportsProjects;

/// The `exports` plugin: the scripts projects whose externs a tool browses and writes manifests of.
pub struct ExportsPlugin;

impl ExportsPlugin {
  /// The plugin, its projects managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::exports::builder()
      .setup(|application, _| {
        application.manage(ExportsProjects::new(
          "exports project",
          &application.state::<SessionScopes>(),
        ));

        Ok(())
      })
      .build()
  }
}
