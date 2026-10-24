use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// Whole-project gamedata checks.
pub struct GamedataPlugin {}

impl GamedataPlugin {
  pub const NAME: &'static str = crate::registry::gamedata::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(xrf_app_core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::gamedata::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::gamedata::specta_builder()
  }
}
