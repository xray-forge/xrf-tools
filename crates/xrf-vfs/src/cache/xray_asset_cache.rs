use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use xrf_error::XrfResult;

use crate::asset::XrayAssetType;
use crate::cache::xray_cache_entries::XrayCacheEntries;
use crate::cache::xray_cache_flights::{XrayCacheFlightClaim, XrayCacheFlights};
use crate::cache::xray_cache_key::XrayCacheKey;
use crate::cache::xray_cache_policy::XrayCachePolicy;
use crate::cache::xray_cache_stats::XrayCacheStats;
use crate::cache::xray_stamped_copy::XrayStampedCopy;
use crate::{XrayFileStamp, XrayRevision};

/// Parsed assets a VFS snapshot and its forks keep, so a second read of the same copy costs a hash lookup.
///
/// # Statistics
///
/// [`Self::get_or_load`] is the only operation that moves `hits` and `misses`, and it moves them by one rule: **a miss
/// is a read that ran the load, and a hit is a read that did not have to.** Their sum is therefore the number of parse
/// requests a run made, which is a property of its inputs. Plain lookups do not count, so inspecting the store cannot
/// change what a run reports about itself.
#[derive(Debug, Default)]
pub struct XrayAssetCache {
  policy: XrayCachePolicy,
  entries: RwLock<XrayCacheEntries>,
  flights: XrayCacheFlights,
  hits: AtomicU64,
  misses: AtomicU64,
  refused: AtomicU64,
  /// The last project revision a change of this lineage took, so two changes branched from one snapshot never share a
  /// revision, and so never a retained value's key.
  issued: AtomicU64,
}

impl XrayAssetCache {
  pub fn new(policy: XrayCachePolicy) -> Self {
    Self {
      policy,
      ..Default::default()
    }
  }

  pub fn get_policy(&self) -> &XrayCachePolicy {
    &self.policy
  }

  /// The revision a change of a snapshot at `current` takes: past `current` and past every revision this lineage issued.
  pub(crate) fn issue_revision(&self, current: XrayRevision) -> XrayRevision {
    let next = |last: u64| last.max(current.get()) + 1;
    let last: u64 = self
      .issued
      .try_update(Ordering::SeqCst, Ordering::SeqCst, |last| Some(next(last)))
      .unwrap_or_else(|last| last);

    XrayRevision::from_count(next(last))
  }

  /// Returns a retained value, if the store is holding one under that identity.
  ///
  /// A plain lookup, outside the accounting: it reports what the store holds without changing what the run reports
  /// about itself. [`Self::get_or_load`] is the read that counts.
  #[cfg(test)]
  pub fn get<T: Send + Sync + 'static>(&self, key: &XrayCacheKey) -> Option<Arc<T>> {
    self.peek(key)
  }

  /// Serves a retained value, or loads it once on behalf of every thread asking for it at the same moment.
  ///
  /// `load` answers with the parsed value, the source length it was parsed from and, for a loose copy, how its file
  /// looked when read, and runs only when nothing is retained. Two threads missing the same retained key produce one
  /// load and the counters of the sequential pair that would have run instead — one miss and one hit — so a report
  /// describes the inputs rather than the schedule.
  ///
  /// # Errors
  ///
  /// Returns whatever `load` answers with. Failures are not retained and are never shared: a waiter whose owner failed
  /// loads again and reports its own error, which is both what a sequential pair would have done and what keeps one
  /// reader's failure from being served to every reader behind it.
  pub fn get_or_load<T, F>(&self, key: XrayCacheKey, kind: XrayAssetType, load: F) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce() -> XrfResult<(T, u64, Option<XrayFileStamp>)>,
  {
    // Coordinating a kind the store will not retain could only turn a second reader's miss into a hit, because nothing
    // would be published for that reader to find.
    if !self.policy.is_allowed(kind) {
      return self.load_uncoordinated(load);
    }

    if let Some(retained) = self.serve_retained::<T>(&key) {
      return Ok(retained);
    }

    match self.flights.claim(&key) {
      // `_guard` is held, not ignored: dropping it at the end of this arm is what releases the waiters. Replacing the
      // binding with `_` would drop it immediately and let every waiter through before anything is published.
      XrayCacheFlightClaim::Owned(_guard) => {
        // Published while this reader was claiming the load, which in sequence would have been a hit, so it is one.
        if let Some(retained) = self.serve_retained::<T>(&key) {
          return Ok(retained);
        }

        let (value, bytes, stamp) = self.load_and_count(load)?;

        self.retain(key, kind, bytes, stamp, Arc::clone(&value));

        Ok(value)
      }
      XrayCacheFlightClaim::Waiting(flight) => {
        flight.wait();

        match self.serve_retained::<T>(&key) {
          Some(retained) => Ok(retained),
          // Nothing was published, so the owner failed and this is the second failing read a sequential pair makes.
          None => self.load_uncoordinated(load),
        }
      }
      XrayCacheFlightClaim::Reentrant => self.load_uncoordinated(load),
    }
  }

  /// Retains a parsed value if the policy allows its kind and the budget has room.
  #[cfg(test)]
  pub fn insert<T: Send + Sync + 'static>(&self, key: XrayCacheKey, kind: XrayAssetType, bytes: u64, value: Arc<T>) {
    self.retain(key, kind, bytes, None, value);
  }

  /// Drops everything retained for one copy, whatever type or revision held it.
  // todo: Index the keys by copy, so a change of many files stops scanning every retained value once per file.
  pub fn forget_copy(&self, opening: u64, path: &str) -> usize {
    self.write_entries().retain(|key, _| !key.is_copy(opening, path))
  }

  /// Every retained value's loose copy among the sources `is_listed` picks by opening, as its read saw the file: one
  /// per value, so a copy two types were parsed from is listed twice.
  pub(crate) fn list_stamped(&self, is_listed: impl Fn(u64) -> bool) -> Vec<XrayStampedCopy> {
    self
      .read_entries()
      .map
      .iter()
      .filter(|(key, _)| is_listed(key.get_opening()))
      .filter_map(|(key, entry)| {
        Some(XrayStampedCopy {
          opening: key.get_opening(),
          path: key.get_path().to_owned(),
          revision: key.get_revision(),
          stamp: entry.stamp?,
        })
      })
      .collect()
  }

  /// Drops everything no caller is holding, and answers how many entries went.
  #[cfg(test)]
  pub fn clean_unreferenced(&self) -> usize {
    self
      .write_entries()
      .retain(|_, entry| Arc::strong_count(&entry.value) > 1)
  }

  pub fn get_stats(&self) -> XrayCacheStats {
    let entries = self.read_entries();

    XrayCacheStats {
      entries: entries.map.len(),
      bytes: entries.bytes,
      hits: self.hits.load(Ordering::Relaxed),
      misses: self.misses.load(Ordering::Relaxed),
      refused: self.refused.load(Ordering::Relaxed),
    }
  }

  /// Answers with a retained value and counts the hit it is.
  fn serve_retained<T: Send + Sync + 'static>(&self, key: &XrayCacheKey) -> Option<Arc<T>> {
    let retained: Arc<T> = self.peek(key)?;

    self.hits.fetch_add(1, Ordering::Relaxed);

    Some(retained)
  }

  /// Runs the load and counts the miss it is.
  ///
  /// # Errors
  ///
  /// Returns whatever `load` answers with. The miss stands: the read happened, whatever it found.
  fn load_and_count<T, F>(&self, load: F) -> XrfResult<(Arc<T>, u64, Option<XrayFileStamp>)>
  where
    T: Send + Sync + 'static,
    F: FnOnce() -> XrfResult<(T, u64, Option<XrayFileStamp>)>,
  {
    self.misses.fetch_add(1, Ordering::Relaxed);

    let (value, bytes, stamp) = load()?;

    Ok((Arc::new(value), bytes, stamp))
  }

  /// Loads for this reader alone, retaining nothing and sharing nothing.
  ///
  /// # Errors
  ///
  /// Returns whatever `load` answers with.
  fn load_uncoordinated<T, F>(&self, load: F) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce() -> XrfResult<(T, u64, Option<XrayFileStamp>)>,
  {
    Ok(self.load_and_count(load)?.0)
  }

  /// Looks without counting, for the callers that decide what the look meant only afterwards.
  fn peek<T: Send + Sync + 'static>(&self, key: &XrayCacheKey) -> Option<Arc<T>> {
    self
      .read_entries()
      .map
      .get(key)
      .and_then(|entry| Arc::clone(&entry.value).downcast::<T>().ok())
  }

  /// Holds a value, subject to the policy and the budget.
  fn retain<T: Send + Sync + 'static>(
    &self,
    key: XrayCacheKey,
    kind: XrayAssetType,
    bytes: u64,
    stamp: Option<XrayFileStamp>,
    value: Arc<T>,
  ) {
    if !self.policy.is_allowed(kind) {
      return;
    }

    let mut entries = self.write_entries();

    if let Some(budget) = self.policy.get_budget() {
      if entries.bytes + bytes > budget {
        entries.evict_for(bytes, budget);
      }

      // Still over after dropping what nobody holds: stop retaining rather than exceed a ceiling that exists to keep a
      // session out of swap. The caller already has the value, so this costs a re-parse later and nothing else.
      if entries.bytes + bytes > budget {
        self.refused.fetch_add(1, Ordering::Relaxed);

        return;
      }
    }

    entries.insert(key, value, bytes, stamp);
  }

  fn read_entries(&self) -> RwLockReadGuard<'_, XrayCacheEntries> {
    self.entries.read().expect("asset cache lock is never poisoned")
  }

  fn write_entries(&self) -> RwLockWriteGuard<'_, XrayCacheEntries> {
    self.entries.write().expect("asset cache lock is never poisoned")
  }
}
