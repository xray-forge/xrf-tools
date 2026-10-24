use std::thread;
use std::time::Duration;

use tauri::Runtime;
use tauri::webview::WebviewWindow;

/// Shows the main window when the document does not, so a webview that never boots still leaves something on screen.
pub struct WindowReveal;

impl WindowReveal {
  /// How long the window stays hidden waiting for the document to reveal itself.
  const TIMEOUT: Duration = Duration::from_secs(1);

  /// Reveal the window after [`Self::TIMEOUT`] unless the document already has.
  pub fn on_timeout<R: Runtime>(window: WebviewWindow<R>) {
    thread::spawn(move || {
      thread::sleep(Self::TIMEOUT);

      match window.is_visible() {
        Ok(true) => (),
        Ok(false) => {
          log::warn!("Revealing main window the document left hidden for {:?}", Self::TIMEOUT);

          if let Err(error) = window.show() {
            log::error!("Failed to reveal main window: {error}");
          }
        }
        Err(error) => log::error!("Failed to read main window visibility: {error}"),
      }
    });
  }
}
