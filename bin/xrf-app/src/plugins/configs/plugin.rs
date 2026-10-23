use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};

use crate::core::assets::AssetMountState;
use crate::plugins::configs::state::ConfigsState;

pub struct ConfigsPlugin {}

impl ConfigsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::configs::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::configs::handler(),
      ))
      .setup(|application, _| {
        let state: ConfigsState = ConfigsState::new("configs");

        // The configs editor's open records every config it reads, so a commit tells whether it went stale.
        application.state::<AssetMountState>().follow(state.clone());
        application.manage(state);

        Ok(())
      })
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::configs::specta_builder()
  }
}
