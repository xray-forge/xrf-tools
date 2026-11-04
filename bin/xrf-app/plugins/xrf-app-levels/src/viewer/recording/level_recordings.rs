use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[cfg(windows)]
use crate::viewer::recording::ViewportRecorder;

/// The level viewer's recording, one at a time: the recorder while it runs, and once it ended by itself until it is
/// stopped and collected. Off Windows nothing records.
#[derive(Clone, Default)]
pub struct LevelRecordings {
  #[cfg(windows)]
  active: Arc<Mutex<Option<ViewportRecorder>>>,
  #[cfg(not(windows))]
  active: Arc<Mutex<Option<()>>>,
}

impl LevelRecordings {
  /// The recording, while there is one.
  #[cfg(windows)]
  pub fn lock(&self) -> MutexGuard<'_, Option<ViewportRecorder>> {
    self.active.lock().unwrap_or_else(PoisonError::into_inner)
  }

  /// Whether a recording runs, none off Windows.
  pub fn is_recording(&self) -> bool {
    #[cfg(windows)]
    {
      self.lock().as_ref().is_some_and(|recorder| !recorder.is_ended())
    }

    #[cfg(not(windows))]
    {
      self.active.lock().unwrap_or_else(PoisonError::into_inner).is_some()
    }
  }
}
