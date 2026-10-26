use tauri::Runtime;
use tauri::webview::WebviewWindow;

/// A window waiting hidden for its page, as its reveal acts on it, so what decides when a window shows is testable
/// without one.
pub(crate) trait HeldWindow: Send {
  /// Shows and focuses the window, logging what failed.
  fn reveal(&self);
}

impl<R: Runtime> HeldWindow for WebviewWindow<R> {
  fn reveal(&self) {
    let label: &str = self.label();

    if let Err(error) = self.show() {
      log::error!("Failed to show window '{label}': {error}");

      return;
    }

    if let Err(error) = self.set_focus() {
      log::error!("Failed to focus window '{label}': {error}");
    }

    log::info!("Revealed window '{label}'");
  }
}
