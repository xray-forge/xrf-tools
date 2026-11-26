use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::LevelSourceResult;
use crate::sound::BurstSoundFrame;

/// Copies a burst's sound frames off the live scene: armed for one burst at a time, every shown level's sink sends it
/// the frames the world marks as a burst's. A clone shares it.
#[derive(Clone, Default)]
pub struct BurstSoundTee {
  armed: Arc<Mutex<Option<Sender<BurstSoundFrame>>>>,
}

impl BurstSoundTee {
  /// Receive every burst frame sent from now until disarmed.
  ///
  /// # Errors
  ///
  /// Returns an error while another burst is armed.
  pub fn arm(&self) -> LevelSourceResult<Receiver<BurstSoundFrame>> {
    let mut armed = self.lock();

    if armed.is_some() {
      return Err(String::from("Another burst records its sound"));
    }

    let (sender, frames) = mpsc::channel();

    *armed = Some(sender);

    Ok(frames)
  }

  /// Stop sending, closing the armed burst's receiver.
  pub fn disarm(&self) {
    *self.lock() = None;
  }

  /// Whether a burst is armed; sinks copy frames only then.
  pub fn is_armed(&self) -> bool {
    self.lock().is_some()
  }

  /// Hand `frame` to the armed burst, if any.
  pub fn send(&self, frame: BurstSoundFrame) {
    if let Some(sender) = self.lock().as_ref() {
      // A dropped burst's receiver is gone; it disarms as it goes.
      let _ = sender.send(frame);
    }
  }

  // The slot is replaced whole, so a poisoned one is still valid.
  fn lock(&self) -> MutexGuard<'_, Option<Sender<BurstSoundFrame>>> {
    self.armed.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
