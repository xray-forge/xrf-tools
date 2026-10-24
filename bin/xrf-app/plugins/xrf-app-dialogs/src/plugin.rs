use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::session::SessionScopes;

use crate::state::DialogProjectState;

pub struct DialogsPlugin {}

impl DialogsPlugin {
  pub const NAME: &'static str = crate::registry::dialogs::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(DialogProjectState::new(
          "dialogs",
          &application.state::<SessionScopes>(),
        ));

        Ok(())
      })
      .invoke_handler(xrf_app_core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::dialogs::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::dialogs::specta_builder()
  }
}
