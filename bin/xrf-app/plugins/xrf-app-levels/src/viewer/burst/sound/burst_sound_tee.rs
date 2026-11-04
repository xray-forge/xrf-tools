use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_app_core::TauriResult;

use crate::viewer::burst::BurstSoundFrame;

/// Where the sound frames a burst draws go besides its scene: armed for one burst at a time, every shown level's sink
/// sends it the frames the world marks as a burst's. A clone shares it.
#[derive(Clone, Default)]
pub struct BurstSoundTee {
  armed: Arc<Mutex<Option<Sender<BurstSoundFrame>>>>,
}

impl BurstSoundTee {
  /// Takes every burst frame sent from now on, until disarmed.
  ///
  /// # Errors
  ///
  /// Returns an error while another burst takes them, as the renderer draws one burst at a time.
  pub fn arm(&self) -> TauriResult<Receiver<BurstSoundFrame>> {
    let mut armed = self.lock();

    if armed.is_some() {
      return Err(String::from("Another burst records its sound"));
    }

    let (sender, frames) = mpsc::channel();

    *armed = Some(sender);

    Ok(frames)
  }

  /// Takes no more frames, ending what the burst armed with receives.
  pub fn disarm(&self) {
    *self.lock() = None;
  }

  /// Whether a burst takes frames, so a sink copies them only then.
  pub fn is_armed(&self) -> bool {
    self.lock().is_some()
  }

  /// Hands `frame` to the burst taking frames, if any is.
  pub fn send(&self, frame: BurstSoundFrame) {
    if let Some(sender) = self.lock().as_ref() {
      // A burst gone takes nothing more; it disarms as it goes.
      let _ = sender.send(frame);
    }
  }

  // The slot changes whole, so one a panic left behind is still whole.
  fn lock(&self) -> MutexGuard<'_, Option<Sender<BurstSoundFrame>>> {
    self.armed.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
