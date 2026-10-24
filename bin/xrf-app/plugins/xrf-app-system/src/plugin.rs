use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};

use crate::diagnostics::MachineProbeState;
use crate::diagnostics::memory::WebviewProcessProbe;

/// Desktop integration that belongs to no editor in particular.
pub struct SystemPlugin;

impl SystemPlugin {
  /// The plugin, its probes managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::system::builder()
      .setup(|application, _| {
        application.manage(MachineProbeState::new());
        application.manage(WebviewProcessProbe::new(application.clone()));

        Ok(())
      })
      .build()
  }
}
