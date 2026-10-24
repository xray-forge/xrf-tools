use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionScopes;

use crate::state::TextureState;

pub struct TexturesPlugin {}

impl TexturesPlugin {
  pub const NAME: &'static str = crate::registry::textures::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(TextureState::new(
          AssetMountState::clone(&application.state::<AssetMountState>()),
          &application.state::<SessionScopes>(),
        ));

        Ok(())
      })
      .invoke_handler(xrf_app_core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::textures::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::textures::specta_builder()
  }
}
