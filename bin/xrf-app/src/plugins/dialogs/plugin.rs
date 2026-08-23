use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};

use crate::plugins::dialogs::state::DialogProjectState;

pub struct DialogsPlugin {}

impl DialogsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::dialogs::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(DialogProjectState::new());

        Ok(())
      })
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::dialogs::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::dialogs::specta_builder()
  }
}
