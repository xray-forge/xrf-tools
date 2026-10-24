use tauri::Runtime;
use tauri::plugin::TauriPlugin;

/// Says where the transport listens, without owning it: the composition root starts it.
pub struct TransportPlugin;

impl TransportPlugin {
  /// The plugin, answering where the transport listens.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::transport::builder().build()
  }
}
