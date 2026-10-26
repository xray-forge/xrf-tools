use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionScopes;

use crate::project::{ConfigsProjects, OpenedConfigsProject};

/// The `configs` plugin: configs projects browsed and followed through commits, and a directory's configs verified,
/// formatted and checked as jobs.
pub struct ConfigsPlugin;

impl ConfigsPlugin {
  /// The plugin, its projects managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::configs::builder()
      .setup(|application, _| {
        // Read again on the pool once a commit moved a config the open read: an installation's configs take a while.
        application.manage(
          ConfigsProjects::new("configs project", &application.state::<SessionScopes>()).with_deferred_derivation(
            &application.state::<AssetMountState>(),
            application.state::<ExecutionState>().get_pool(),
            OpenedConfigsProject::derive,
          ),
        );

        Ok(())
      })
      .build()
  }
}
