use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The running-work surface: what is going on, and asking it to stop.
pub struct JobsPlugin {}

impl JobsPlugin {
  pub const NAME: &'static str = crate::registry::jobs::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(crate::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::jobs::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::jobs::specta_builder()
  }
}
