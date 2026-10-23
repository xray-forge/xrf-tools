use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The scopes surface: binding an opening to the tool that asked for it, and releasing what a tool or a page left.
pub struct SessionsPlugin {}

impl SessionsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::sessions::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::sessions::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::sessions::specta_builder()
  }
}
