use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, ThreadId};

use crate::cache::xray_cache_key::XrayCacheKey;

/// The loads a store has in progress, so concurrent requesters of one key produce one load instead of one each.
///
/// This is coordination alone. Whether a key is worth coordinating, what a wait means for reported statistics, and what
/// a released waiter does next are the store's questions, answered in [`crate::XrayAssetCache`].
#[derive(Debug, Default)]
pub(crate) struct XrayCacheFlights {
  in_progress: Mutex<HashMap<XrayCacheKey, Arc<XrayCacheFlight>>>,
}

impl XrayCacheFlights {
  /// Says what this caller is to a key, admitting exactly one owner per key at a time.
  pub(crate) fn claim(&self, key: &XrayCacheKey) -> XrayCacheFlightClaim<'_> {
    let mut in_progress = self.lock();

    match in_progress.get(key) {
      // Waiting on a flight this thread owns would be waiting on itself, so a load that re-enters the store for the key
      // it is already loading proceeds uncoordinated rather than hanging a sweep with nothing to show for it.
      Some(flight) if flight.owner == thread::current().id() => XrayCacheFlightClaim::Reentrant,
      Some(flight) => XrayCacheFlightClaim::Waiting(Arc::clone(flight)),
      None => {
        in_progress.insert(key.clone(), Arc::new(XrayCacheFlight::new()));

        XrayCacheFlightClaim::Owned(XrayCacheFlightGuard {
          flights: self,
          key: key.clone(),
        })
      }
    }
  }

  /// Ends one flight and releases everyone waiting on it.
  fn release(&self, key: &XrayCacheKey) {
    if let Some(flight) = self.lock().remove(key) {
      flight.finish();
    }
  }

  fn lock(&self) -> MutexGuard<'_, HashMap<XrayCacheKey, Arc<XrayCacheFlight>>> {
    self.in_progress.lock().expect("cache flights lock is never poisoned")
  }
}

/// What one caller is to a key it wants loaded.
pub(crate) enum XrayCacheFlightClaim<'a> {
  /// This caller performs the load. Dropping the guard releases the waiters, however the load ended.
  Owned(XrayCacheFlightGuard<'a>),
  /// Another thread is loading this key, and [`XrayCacheFlight::wait`] returns once it has finished.
  Waiting(Arc<XrayCacheFlight>),
  /// This thread is already loading this key further up its own stack.
  Reentrant,
}

/// One load in progress, and the thread performing it.
#[derive(Debug)]
pub(crate) struct XrayCacheFlight {
  owner: ThreadId,
  is_finished: Mutex<bool>,
  finished: Condvar,
}

impl XrayCacheFlight {
  /// Blocks until the owner is done, returning at once when it already is.
  ///
  /// Safe against a finish that lands between a caller receiving this flight and waiting on it: the owner sets the flag
  /// before notifying, and a waiter that arrives afterwards sees the flag rather than a notification it missed.
  pub(crate) fn wait(&self) {
    let mut is_finished = self.lock();

    while !*is_finished {
      is_finished = self.finished.wait(is_finished).expect("cache flight is never poisoned");
    }
  }

  fn new() -> Self {
    Self {
      owner: thread::current().id(),
      is_finished: Mutex::new(false),
      finished: Condvar::new(),
    }
  }

  fn finish(&self) {
    *self.lock() = true;

    self.finished.notify_all();
  }

  fn lock(&self) -> MutexGuard<'_, bool> {
    self.is_finished.lock().expect("cache flight is never poisoned")
  }
}

/// Ends the owner's flight however its load turns out, including an unwinding panic.
///
/// Without this a panicking load would leave its waiters blocked for the life of the process, which is the one failure
/// of coordinated loading that a caller could not diagnose from anything it can observe.
pub(crate) struct XrayCacheFlightGuard<'a> {
  flights: &'a XrayCacheFlights,
  key: XrayCacheKey,
}

impl Drop for XrayCacheFlightGuard<'_> {
  fn drop(&mut self) {
    self.flights.release(&self.key);
  }
}
