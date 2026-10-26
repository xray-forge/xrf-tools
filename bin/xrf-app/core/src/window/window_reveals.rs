use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

use tauri::Runtime;
use tauri::webview::WebviewWindow;

use crate::window::held_window::HeldWindow;

/// The windows built hidden that wait for their page to put them on screen, by label. The first reveal of a window shows
/// and focuses it; a later one, from a page reloaded over it, finds nothing left to do.
#[derive(Clone, Default)]
pub struct WindowReveals {
  held: Arc<Mutex<HashMap<String, Box<dyn HeldWindow>>>>,
}

impl WindowReveals {
  /// How long a window stays hidden waiting for its page.
  const TIMEOUT: Duration = Duration::from_secs(1);

  /// Holds `window` hidden until its page reveals it, and reveals it anyway once [`Self::TIMEOUT`] has passed, so a page
  /// that never boots still leaves something on screen.
  pub fn hold<R: Runtime>(&self, window: WebviewWindow<R>) {
    self.hold_for(window.label().to_owned(), Box::new(window), Self::TIMEOUT);
  }

  /// Shows and focuses the window labelled `label`; whether it was still waiting.
  pub fn reveal(&self, label: &str) -> bool {
    let Some(window) = self.take(label) else {
      return false;
    };

    window.reveal();

    true
  }

  /// [`Self::hold`] for any held window under `label`, revealed anyway once `timeout` has passed.
  pub(crate) fn hold_for(&self, label: String, window: Box<dyn HeldWindow>, timeout: Duration) {
    self.lock().insert(label.clone(), window);

    let reveals: Self = self.clone();
    let waited: String = label.clone();
    let spawned = thread::Builder::new().name("xrf-window-reveal".into()).spawn(move || {
      thread::sleep(timeout);

      if let Some(window) = reveals.take(&waited) {
        log::warn!("Revealing window '{waited}' its page left hidden for {timeout:?}");

        window.reveal();
      }
    });

    if let Err(error) = spawned {
      log::error!("Window '{label}' cannot wait for its page, so is revealed now: {error}");

      self.reveal(&label);
    }
  }

  fn take(&self, label: &str) -> Option<Box<dyn HeldWindow>> {
    self.lock().remove(label)
  }

  /// The held windows, whatever a panicking caller left: an entry is only ever added or removed whole.
  fn lock(&self) -> MutexGuard<'_, HashMap<String, Box<dyn HeldWindow>>> {
    self.held.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
