use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

pub struct AssetsPlugin {}

impl AssetsPlugin {
  pub const NAME: &'static str = crate::ipc::registry::assets::NAME;

  /// Exposes the asset roots over IPC, without owning it.
  ///
  /// The one plugin with no state of its own: the roots it reads through is `core/`'s, managed by the composition root
  /// so the domains that share it do not depend on this plugin having been initialized.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(Self::NAME)
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::assets::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::assets::specta_builder()
  }
}
