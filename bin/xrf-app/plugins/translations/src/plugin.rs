use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionScopes;

use crate::state::TranslationProjectState;

pub struct TranslationsPlugin {}

impl TranslationsPlugin {
  pub const NAME: &'static str = crate::registry::translations::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .setup(|application, _| {
        let state: TranslationProjectState = TranslationProjectState::new(&application.state::<SessionScopes>());

        // An open project records every file it read and reads them again when a commit moved one: a save among them.
        application
          .state::<AssetMountState>()
          .add_follower(state.projects.clone());
        application.manage(state);

        Ok(())
      })
      .invoke_handler(xrf_app_core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::translations::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::translations::specta_builder()
  }
}
