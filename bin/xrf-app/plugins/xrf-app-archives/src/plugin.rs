use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};
use xrf_app_core::session::SessionScopes;

use crate::browse::ArchiveBrowseState;

pub struct ArchivesPlugin {}

impl ArchivesPlugin {
  pub const NAME: &'static str = crate::registry::archives::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(ArchiveBrowseState::new(
          "archive browse",
          &application.state::<SessionScopes>(),
        ));

        Ok(())
      })
      .invoke_handler(xrf_app_core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::archives::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::archives::specta_builder()
  }
}
