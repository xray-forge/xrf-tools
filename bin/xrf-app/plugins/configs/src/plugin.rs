use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionScopes;

use crate::opened_configs_project::OpenedConfigsProject;
use crate::state::ConfigsState;

pub struct ConfigsPlugin {}

impl ConfigsPlugin {
  pub const NAME: &'static str = crate::registry::configs::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(xrf_app_core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::configs::handler(),
      ))
      .setup(|application, _| {
        // Read again on the pool once a commit moved a config the open read: an installation's configs take a while.
        let state: ConfigsState = ConfigsState::new("configs", &application.state::<SessionScopes>())
          .with_deferred_derivation(
            application.state::<ExecutionState>().get_pool(),
            OpenedConfigsProject::derive,
          );

        application.state::<AssetMountState>().add_follower(state.clone());
        application.manage(state);

        Ok(())
      })
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::configs::specta_builder()
  }
}
