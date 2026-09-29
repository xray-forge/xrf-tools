use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};

use crate::plugins::environment::state::EnvironmentState;

pub struct EnvironmentPlugin {}

impl EnvironmentPlugin {
  pub const NAME: &'static str = crate::ipc::registry::environment::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::environment::handler(),
      ))
      .setup(|application, _| {
        application.manage(EnvironmentState::new());

        Ok(())
      })
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::environment::specta_builder()
  }
}
