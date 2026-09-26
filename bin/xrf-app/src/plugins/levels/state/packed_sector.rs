use std::sync::Mutex;

/// One packed sector parked between the call that described it and the call that serves its bytes.
pub struct PackedSector {
  /// Which sector of the level the bytes are.
  pub sector: u32,
  /// Taken by the read that serves it rather than copied.
  pub buffer: Mutex<Option<Vec<u8>>>,
}

impl PackedSector {
  /// Takes the bytes, leaving nothing behind for a second read to serve.
  ///
  /// # Errors
  ///
  /// Returns an error when the lock is poisoned, or when the bytes have already been served.
  pub fn take_buffer(&self) -> Result<Vec<u8>, String> {
    self
      .buffer
      .lock()
      .map_err(|error| format!("Failed to read the packed sector: {error}"))?
      .take()
      .ok_or_else(|| String::from("The packed sector's bytes have already been read"))
  }
}
