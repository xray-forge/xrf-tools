use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use xrf_error::XrfResult;

use crate::asset::XrayAssetType;
use crate::cache::xray_cache_flights::{XrayCacheFlightClaim, XrayCacheFlights};
use crate::cache::xray_cache_key::XrayCacheKey;
use crate::cache::xray_cache_policy::XrayCachePolicy;
use crate::cache::xray_cache_stats::XrayCacheStats;
use crate::vfs::XrayLookupScope;

#[derive(Debug)]
struct XrayCacheEntry {
  value: Arc<dyn Any + Send + Sync>,
  bytes: u64,
}

/// Parsed assets a mounted world keeps, so a second read of the same asset costs a hash lookup.
///
/// Lives on the mount rather than in a global, which is what keeps two open projects from serving each other's assets
/// and what makes the lifetime obvious — the store dies with the world it describes. The engine reaches the same
/// arrangement from the other end: `motions_container` in `xray-16` is global because a running game mounts exactly one
/// world, and it checks its own map before opening a file for the reason this type exists.
///
/// Retention is policy, never a property of the call site. A kind the policy excludes flows through the identical read,
/// parses, and simply is not kept, so enabling it later changes no code.
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
  entries: RwLock<HashMap<XrayCacheKey, XrayCacheEntry>>,
  flights: XrayCacheFlights,
  hits: AtomicU64,
  misses: AtomicU64,
  refused: AtomicU64,
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

  /// Returns a retained value, if this world is holding one under that identity.
  ///
  /// A plain lookup, outside the accounting: it reports what the store holds without changing what the run reports
  /// about itself. [`Self::get_or_load`] is the read that counts.
  pub fn get<T: Send + Sync + 'static>(&self, scope: &XrayLookupScope, path: &str) -> Option<Arc<T>> {
    self.peek(&XrayCacheKey::of::<T>(scope, path))
  }

  /// Serves a retained value, or loads it once on behalf of every thread asking for it at the same moment.
  ///
  /// `load` answers with the parsed value and the source length it was parsed from, and runs only when nothing is
  /// retained. Two threads missing the same retained key produce one load and the counters of the sequential pair that
  /// would have run instead — one miss and one hit — so a report describes the inputs rather than the schedule.
  ///
  /// # Errors
  ///
  /// Returns whatever `load` answers with. Failures are not retained and are never shared: a waiter whose owner failed
  /// loads again and reports its own error, which is both what a sequential pair would have done and what keeps one
  /// reader's failure from being served to every reader behind it.
  pub fn get_or_load<T, F>(
    &self,
    scope: &XrayLookupScope,
    path: &str,
    kind: XrayAssetType,
    load: F,
  ) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce() -> XrfResult<(T, u64)>,
  {
    // Coordinating a kind the store will not retain could only turn a second reader's miss into a hit, because nothing
    // would be published for that reader to find.
    if !self.policy.is_allowed(kind) {
      return self.load_uncoordinated(load);
    }

    let key: XrayCacheKey = XrayCacheKey::of::<T>(scope, path);

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

        let (value, bytes) = self.load_and_count(load)?;

        self.retain(key, kind, bytes, Arc::clone(&value));

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
  ///
  /// `bytes` is the source length the value was parsed from, which stands in for its parsed size: within an order of
  /// magnitude of the real thing, and free to obtain, where a real measurement would need every format crate to
  /// describe itself.
  pub fn insert<T: Send + Sync + 'static>(
    &self,
    scope: &XrayLookupScope,
    path: &str,
    kind: XrayAssetType,
    bytes: u64,
    value: Arc<T>,
  ) {
    self.retain(XrayCacheKey::of::<T>(scope, path), kind, bytes, value);
  }

  /// Drops everything retained for one logical path, whatever type or scope held it.
  pub fn forget(&self, logical_path: &str) -> usize {
    self.drop_entries(|key, _| key.get_path() != logical_path)
  }

  /// Drops everything no caller is holding, and answers how many entries went.
  ///
  /// The refcount is the liveness signal, so this is safe at any moment: an entry with one reference is held by this
  /// store alone. `xray-16` spells the same idea `motions_container::clean(false)` and calls it between levels.
  pub fn clean_unreferenced(&self) -> usize {
    self.drop_entries(|_, entry| Arc::strong_count(&entry.value) > 1)
  }

  /// Drops everything, including values callers still hold — they keep their own references.
  pub fn clear(&self) {
    self.write_entries().clear();
  }

  pub fn get_stats(&self) -> XrayCacheStats {
    let entries = self.read_entries();

    XrayCacheStats {
      entries: entries.len(),
      bytes: Self::retained_bytes(&entries),
      hits: self.hits.load(Ordering::Relaxed),
      misses: self.misses.load(Ordering::Relaxed),
      refused: self.refused.load(Ordering::Relaxed),
    }
  }

  /// Answers with a retained value and counts the hit it is.
  ///
  /// The only place a hit is counted, so a hit means exactly one thing: this reader was spared a load.
  fn serve_retained<T: Send + Sync + 'static>(&self, key: &XrayCacheKey) -> Option<Arc<T>> {
    let retained: Arc<T> = self.peek(key)?;

    self.hits.fetch_add(1, Ordering::Relaxed);

    Some(retained)
  }

  /// Runs the load and counts the miss it is.
  ///
  /// The only place `load` runs, so a miss means exactly one thing: this reader parsed. Every path that declines to
  /// coordinate reaches its load through here, which is what keeps the two counters describing one rule.
  ///
  /// # Errors
  ///
  /// Returns whatever `load` answers with. The miss stands: the read happened, whatever it found.
  fn load_and_count<T, F>(&self, load: F) -> XrfResult<(Arc<T>, u64)>
  where
    T: Send + Sync + 'static,
    F: FnOnce() -> XrfResult<(T, u64)>,
  {
    self.misses.fetch_add(1, Ordering::Relaxed);

    let (value, bytes) = load()?;

    Ok((Arc::new(value), bytes))
  }

  /// Loads for this reader alone, retaining nothing and sharing nothing.
  ///
  /// What an excluded kind, a re-entrant load, and a released waiter whose owner failed all do, for one reason: none of
  /// them can be answered from what the store holds, and each is a read that a sequential run would also have made.
  ///
  /// # Errors
  ///
  /// Returns whatever `load` answers with.
  fn load_uncoordinated<T, F>(&self, load: F) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce() -> XrfResult<(T, u64)>,
  {
    Ok(self.load_and_count(load)?.0)
  }

  /// Looks without counting, for the callers that decide what the look meant only afterwards.
  fn peek<T: Send + Sync + 'static>(&self, key: &XrayCacheKey) -> Option<Arc<T>> {
    self
      .read_entries()
      .get(key)
      .and_then(|entry| Arc::clone(&entry.value).downcast::<T>().ok())
  }

  /// Holds a value, subject to the policy and the budget.
  fn retain<T: Send + Sync + 'static>(&self, key: XrayCacheKey, kind: XrayAssetType, bytes: u64, value: Arc<T>) {
    if !self.policy.is_allowed(kind) {
      return;
    }

    let mut entries = self.write_entries();

    if let Some(budget) = self.policy.get_budget() {
      if Self::retained_bytes(&entries) + bytes > budget {
        entries.retain(|_, entry| Arc::strong_count(&entry.value) > 1);
      }

      // Still over after dropping what nobody holds: stop retaining rather than exceed a ceiling that exists to keep a
      // session out of swap. The caller already has the value, so this costs a re-parse later and nothing else.
      if Self::retained_bytes(&entries) + bytes > budget {
        self.refused.fetch_add(1, Ordering::Relaxed);

        return;
      }
    }

    entries.insert(key, XrayCacheEntry { value, bytes });
  }

  /// Keeps the entries `keep` admits, and answers how many went.
  ///
  /// Shared by the two operations that drop by a rule; whole-store [`Self::clear`] states itself.
  fn drop_entries<F>(&self, keep: F) -> usize
  where
    F: FnMut(&XrayCacheKey, &mut XrayCacheEntry) -> bool,
  {
    let mut entries = self.write_entries();
    let before: usize = entries.len();

    entries.retain(keep);

    before - entries.len()
  }

  fn read_entries(&self) -> RwLockReadGuard<'_, HashMap<XrayCacheKey, XrayCacheEntry>> {
    self.entries.read().expect("asset cache lock is never poisoned")
  }

  fn write_entries(&self) -> RwLockWriteGuard<'_, HashMap<XrayCacheKey, XrayCacheEntry>> {
    self.entries.write().expect("asset cache lock is never poisoned")
  }

  fn retained_bytes(entries: &HashMap<XrayCacheKey, XrayCacheEntry>) -> u64 {
    entries.values().map(|entry| entry.bytes).sum()
  }
}
