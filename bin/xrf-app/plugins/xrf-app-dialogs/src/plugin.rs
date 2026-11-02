use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::session::SessionScopes;

use crate::DialogProjects;

/// The `dialogs` plugin: the dialog trees a tool browses, each dialog read with its phrases.
pub struct DialogsPlugin;

impl DialogsPlugin {
  /// The plugin, its projects managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::dialogs::builder()
      .setup(|application, _| {
        application.manage(DialogProjects::new(
          "dialogs project",
          &application.state::<SessionScopes>(),
        ));

        Ok(())
      })
      .build()
  }
}
