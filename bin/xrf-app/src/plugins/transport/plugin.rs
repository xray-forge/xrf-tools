use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

pub struct TransportPlugin {}

impl TransportPlugin {
  pub const NAME: &'static str = crate::ipc::registry::transport::NAME;

  /// Says where the transport listens, without owning it: the composition root starts it.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::transport::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::transport::specta_builder()
  }
}
