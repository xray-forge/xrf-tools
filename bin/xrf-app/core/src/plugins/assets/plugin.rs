use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

pub struct AssetsPlugin {}

impl AssetsPlugin {
  pub const NAME: &'static str = crate::registry::assets::NAME;

  /// Exposes the asset roots over IPC, without owning it.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .invoke_handler(crate::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::assets::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::assets::specta_builder()
  }
}
