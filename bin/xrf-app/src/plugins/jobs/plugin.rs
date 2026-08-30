use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// The running-work surface: what is going on, and asking it to stop.
///
/// A domain of its own rather than a command on each plugin that has long work, because identity, exclusion and
/// cancellation are the same three questions whatever the work is. The registry it answers from is shared shell
/// (`core::jobs`) and managed at the composition root, so any domain's command registers with it without depending on
/// this plugin.
pub struct JobsPlugin {}

impl JobsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::jobs::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
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
