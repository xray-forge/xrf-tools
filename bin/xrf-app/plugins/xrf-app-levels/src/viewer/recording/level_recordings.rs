use std::sync::{Arc, Mutex, TryLockError};
#[cfg(windows)]
use std::sync::{MutexGuard, PoisonError};

use xrf_app_render::WorldState;
#[cfg(windows)]
use xrf_utils::format_path;

#[cfg(windows)]
use crate::viewer::recording::ViewportRecorder;

/// The level viewer's recording, one at a time: the recorder while it runs, and once it ended by itself until it is
/// stopped and collected, or its window goes. While a recorder is in the slot, the world hears its viewport. Off Windows
/// nothing records.
#[derive(Clone, Default)]
pub(crate) struct LevelRecordings {
  #[cfg(windows)]
  active: Arc<Mutex<Option<ViewportRecorder>>>,
  #[cfg(not(windows))]
  active: Arc<Mutex<Option<()>>>,
}

impl LevelRecordings {
  /// The recording, while there is one. A start holds this for as long as its writer takes to open, so it is taken on
  /// the pool, never on the async runtime.
  #[cfg(windows)]
  pub(crate) fn lock(&self) -> MutexGuard<'_, Option<ViewportRecorder>> {
    self.active.lock().unwrap_or_else(PoisonError::into_inner)
  }

  /// Whether a recording runs or is starting, none off Windows. Never waits, so the async runtime may ask.
  pub(crate) fn is_recording(&self) -> bool {
    let active = match self.active.try_lock() {
      Ok(active) => active,
      Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
      // A start holds the slot while its recorder opens.
      Err(TryLockError::WouldBlock) => return true,
    };

    #[cfg(windows)]
    {
      active.as_ref().is_some_and(|recorder| !recorder.is_ended())
    }

    #[cfg(not(windows))]
    {
      active.is_some()
    }
  }

  /// Stops and finishes the recording the window labelled `window` started, as its page loads again or the window
  /// goes: no page is left to stop it or hear how it ended, so the outcome goes to the log, and the world hears the
  /// focused viewport again.
  pub(crate) fn release_window(&self, window: &str, world: &WorldState) {
    #[cfg(windows)]
    {
      let released: Option<ViewportRecorder> = {
        let mut active: MutexGuard<'_, Option<ViewportRecorder>> = self.lock();

        if active.as_ref().is_some_and(|recorder| recorder.get_window() == window) {
          world.lock().pin_listener(None);
          active.take()
        } else {
          None
        }
      };

      if let Some(recorder) = released {
        let label: String = window.to_owned();
        let finished = std::thread::Builder::new()
          .name(String::from("xrf-recording-release"))
          .spawn(move || match recorder.stop() {
            Ok(outcome) => log::info!(
              "The recording of window '{label}' finished as the window went: {}",
              format_path(&outcome.file)
            ),
            Err(error) => log::warn!("The recording of window '{label}' ended as the window went: {error}"),
          });

        if let Err(error) = finished {
          log::warn!("The recording of window '{window}' was not finished: {error}");
        }
      }
    }

    #[cfg(not(windows))]
    {
      let _ = (window, world);
    }
  }
}
