use std::sync::{Arc, Mutex, MutexGuard};

use indexmap::IndexMap;

use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::levels::state::packed_sector::PackedSector;

/// Packs parked at once before one of them is served.
const PARKED_LIMIT: usize = 16;

/// The sectors packed and not yet served, one entry per read that asked for one.
pub struct PackedSectors {
  parked: Mutex<IndexMap<SessionId, Arc<PackedSector>>>,
}

impl PackedSectors {
  pub fn new() -> Self {
    Self {
      parked: Mutex::new(IndexMap::new()),
    }
  }

  /// Parks one pack for the read that asked for it.
  ///
  /// # Errors
  ///
  /// Returns an error when the lock is poisoned.
  pub fn park(&self, id: SessionId, packed: PackedSector) -> TauriResult<Arc<PackedSector>> {
    let parked: Arc<PackedSector> = Arc::new(packed);

    let mut held: MutexGuard<IndexMap<SessionId, Arc<PackedSector>>> = self.lock()?;

    held.insert(id, Arc::clone(&parked));

    // Oldest first, which is the one whose read is least likely to still be coming.
    while held.len() > PARKED_LIMIT {
      held.shift_remove_index(0);
    }

    drop(held);

    Ok(parked)
  }

  /// Takes the pack a read was described, leaving nothing behind.
  ///
  /// # Errors
  ///
  /// Returns an error when the lock is poisoned, or when no pack answers to that read.
  pub fn take(&self, id: SessionId) -> TauriResult<Arc<PackedSector>> {
    self
      .lock()?
      .shift_remove(&id)
      .ok_or_else(|| String::from("The level sector session has changed or is closed"))
  }

  fn lock(&self) -> TauriResult<MutexGuard<'_, IndexMap<SessionId, Arc<PackedSector>>>> {
    self
      .parked
      .lock()
      .map_err(|error| format!("The level sector session is unavailable: {error}"))
  }
}
