use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The running-work surface: what is going on, and asking it to stop.
pub struct JobsPlugin {}

impl JobsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::jobs::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::jobs::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::jobs::specta_builder()
  }
}
