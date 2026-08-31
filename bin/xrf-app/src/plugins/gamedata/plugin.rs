use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// Whole-project gamedata checks.
///
/// Stateless: a verification opens the project it was given, answers, and lets it go. Nothing is parked between
/// commands, so there is no state slot a later call could find stale.
pub struct GamedataPlugin {}

impl GamedataPlugin {
  pub const NAME: &'static str = crate::ipc::registry::gamedata::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new(Self::NAME)
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::gamedata::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::gamedata::specta_builder()
  }
}
