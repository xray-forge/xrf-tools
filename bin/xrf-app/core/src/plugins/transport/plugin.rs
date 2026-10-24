use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

pub struct TransportPlugin {}

impl TransportPlugin {
  pub const NAME: &'static str = crate::registry::transport::NAME;

  /// Says where the transport listens, without owning it: the composition root starts it.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .invoke_handler(crate::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::transport::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::transport::specta_builder()
  }
}
