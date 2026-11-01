use tauri::Runtime;
use tauri::webview::WebviewWindow;

use crate::window::window_transitions::WindowTransitions;

/// A window waiting hidden for its page, as its reveal acts on it, so what decides when a window shows is testable
/// without one.
pub(crate) trait HeldWindow: Send {
  /// Shows and focuses the window, logging what failed.
  fn reveal(&self);
}

impl<R: Runtime> HeldWindow for WebviewWindow<R> {
  fn reveal(&self) {
    let label: &str = self.label();

    WindowTransitions::disable(self);

    if let Err(error) = self.show() {
      log::error!("Failed to show window '{label}': {error}");
      WindowTransitions::enable(self);

      return;
    }

    // Queued behind the show on the event loop, so the window is shown, and its animation decided, before it may
    // animate again.
    let shown: WebviewWindow<R> = self.clone();

    if let Err(error) = self.run_on_main_thread(move || WindowTransitions::enable(&shown)) {
      log::error!("Failed to let window '{label}' animate again once shown: {error}");
    }

    if let Err(error) = self.set_focus() {
      log::error!("Failed to focus window '{label}': {error}");
    }

    log::info!("Revealed window '{label}'");
  }
}
