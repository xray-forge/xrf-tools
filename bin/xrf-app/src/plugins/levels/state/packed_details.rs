use std::sync::{Mutex, MutexGuard};

use crate::core::session::SessionId;
use crate::core::types::TauriResult;

/// The pack parked, and the read it is for.
type Parked = Option<(SessionId, Vec<u8>)>;

/// The level's grass as packed by an `open_details` and not yet served: one pack, for the read it was described to.
pub struct PackedDetails {
  parked: Mutex<Parked>,
}

impl PackedDetails {
  pub fn new() -> Self {
    Self {
      parked: Mutex::new(None),
    }
  }

  /// Parks the pack for the read that asked for it, in place of any pack not yet served.
  ///
  /// # Errors
  ///
  /// Returns an error when the lock is poisoned.
  pub fn park(&self, id: SessionId, bytes: Vec<u8>) -> TauriResult {
    *self.lock()? = Some((id, bytes));

    Ok(())
  }

  /// Takes the pack a read was described, leaving nothing behind.
  ///
  /// # Errors
  ///
  /// Returns an error when the lock is poisoned, or when no pack answers to that read.
  pub fn take(&self, id: SessionId) -> TauriResult<Vec<u8>> {
    let mut held: MutexGuard<Parked> = self.lock()?;

    match held.take() {
      Some((parked, bytes)) if parked == id => Ok(bytes),
      other => {
        *held = other;

        Err(String::from("The level details session has changed or is closed"))
      }
    }
  }

  fn lock(&self) -> TauriResult<MutexGuard<'_, Parked>> {
    self
      .parked
      .lock()
      .map_err(|error| format!("The level details session is unavailable: {error}"))
  }
}
