use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use crate::plugins::textures::state::TextureState;

pub struct TexturesPlugin {}

impl TexturesPlugin {
  pub const NAME: &'static str = crate::ipc::registry::textures::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(TextureState::new());

        Ok(())
      })
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::textures::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::textures::specta_builder()
  }
}
