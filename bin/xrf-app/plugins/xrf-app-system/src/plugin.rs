use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};

use crate::diagnostics::MachineProbeState;
use crate::diagnostics::memory::WebviewProcessProbe;

/// Desktop integration that belongs to no editor in particular.
pub struct SystemPlugin {}

impl SystemPlugin {
  pub const NAME: &'static str = crate::registry::system::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    tauri::plugin::Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(MachineProbeState::new());
        application.manage(WebviewProcessProbe::new(application.clone()));

        Ok(())
      })
      .invoke_handler(xrf_app_core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::system::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::system::specta_builder()
  }
}
