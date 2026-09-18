use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use crate::plugins::levels::state::LevelState;

pub struct LevelsPlugin {}

impl LevelsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::levels::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(LevelState::new());

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
