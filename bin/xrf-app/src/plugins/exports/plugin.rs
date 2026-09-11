use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};

use crate::plugins::exports::state::ExportsProjectState;

pub struct ExportsPlugin {}

impl ExportsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::exports::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(ExportsProjectState::new("exports"));

        Ok(())
      })
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::exports::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::exports::specta_builder()
  }
}
