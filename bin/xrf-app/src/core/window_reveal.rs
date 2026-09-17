//! Showing the main window when the document does not, so a webview that never boots still leaves something on screen.

use std::thread;
use std::time::Duration;

use tauri::Runtime;
use tauri::webview::WebviewWindow;

/// How long the window stays hidden waiting for the document to reveal itself.
const REVEAL_TIMEOUT: Duration = Duration::from_secs(1);

/// Reveal the window after [`REVEAL_TIMEOUT`] unless the document already has.
pub fn reveal_window_on_timeout<R: Runtime>(window: WebviewWindow<R>) {
  thread::spawn(move || {
    thread::sleep(REVEAL_TIMEOUT);

    match window.is_visible() {
      Ok(true) => (),
      Ok(false) => {
        log::warn!("Revealing main window the document left hidden for {REVEAL_TIMEOUT:?}");

        if let Err(error) = window.show() {
          log::error!("Failed to reveal main window: {error}");
        }
      }
      Err(error) => log::error!("Failed to read main window visibility: {error}"),
    }
  });
}
