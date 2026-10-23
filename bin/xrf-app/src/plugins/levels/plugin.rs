use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::plugins::levels::state::LevelState;

pub struct LevelsPlugin {}

impl LevelsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::levels::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .setup(|application, _| {
        let state: LevelState = LevelState::new(application.state::<ExecutionState>().get_pool());

        // An open level records what it and its parts read, so a commit carries the parts it did not touch and reads
        // the level again where it moved one of its own files.
        application.state::<AssetMountState>().follow(state.selected.clone());
        application.manage(state);

        Ok(())
      })
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::levels::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::levels::specta_builder()
  }
}
