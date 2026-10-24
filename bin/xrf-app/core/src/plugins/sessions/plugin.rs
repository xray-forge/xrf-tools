use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The scopes surface: binding an opening to the tool that asked for it, and releasing what a tool or a page left.
pub struct SessionsPlugin {}

impl SessionsPlugin {
  pub const NAME: &'static str = crate::registry::sessions::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(crate::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::sessions::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::sessions::specta_builder()
  }
}
