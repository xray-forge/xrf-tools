use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::asset::XrayAssetType;
use crate::cache::xray_cache_policy::XrayCachePolicy;
use crate::cache::xray_cache_stats::XrayCacheStats;
use crate::vfs::XrayLookupScope;

/// What identifies a retained value.
///
/// The scope belongs in the key because a logical path does not name bytes on its own: a scope narrowing to some mounts
/// can resolve the same path to a lower-priority copy, and an application may hold several scopes against one mount set
/// at a time. The type belongs in it so one path may hold both a whole parsed file and a cheaper projection of it.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct XrayCacheKey {
  type_id: TypeId,
  scope: XrayLookupScope,
  path: String,
}

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
#[derive(Debug, Default)]
pub struct XrayAssetCache {
  policy: XrayCachePolicy,
  entries: RwLock<HashMap<XrayCacheKey, XrayCacheEntry>>,
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
  /// Read-only on the hit path by design: retention order is deliberately not tracked, because maintaining it would
  /// turn every hit into a write and serialize the parallel sweeps this exists to speed up.
  pub fn get<T: Send + Sync + 'static>(&self, scope: &XrayLookupScope, path: &str) -> Option<Arc<T>> {
    let key: XrayCacheKey = Self::key::<T>(scope, path);
    let entries = self.entries.read().expect("asset cache lock is never poisoned");
    let value: Option<Arc<T>> = entries
      .get(&key)
      .and_then(|entry| Arc::clone(&entry.value).downcast::<T>().ok());

    if value.is_some() {
      self.hits.fetch_add(1, Ordering::Relaxed);
    } else {
      self.misses.fetch_add(1, Ordering::Relaxed);
    }

    value
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
    if !self.policy.is_allowed(kind) {
      return;
    }

    let key: XrayCacheKey = Self::key::<T>(scope, path);
    let mut entries = self.entries.write().expect("asset cache lock is never poisoned");

    if let Some(budget) = self.policy.get_budget() {
      if Self::retained_bytes(&entries) + bytes > budget {
        Self::drop_unreferenced(&mut entries);
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

  /// Drops everything retained for one logical path, whatever type or scope held it.
  ///
  /// Called when a write changes that path's bytes. Deliberately blunt: only a resolve per scope could say which scopes
  /// were serving the mount that was written, and a write is rare where a parse is not.
  pub fn forget(&self, logical_path: &str) -> usize {
    let mut entries = self.entries.write().expect("asset cache lock is never poisoned");
    let before: usize = entries.len();

    entries.retain(|key, _| key.path != logical_path);

    before - entries.len()
  }

  /// Drops everything no caller is holding, and answers how many entries went.
  ///
  /// The refcount is the liveness signal, so this is safe at any moment: an entry with one reference is held by this
  /// store alone. `xray-16` spells the same idea `motions_container::clean(false)` and calls it between levels.
  pub fn clean_unreferenced(&self) -> usize {
    let mut entries = self.entries.write().expect("asset cache lock is never poisoned");
    let before: usize = entries.len();

    Self::drop_unreferenced(&mut entries);

    before - entries.len()
  }

  /// Drops everything, including values callers still hold — they keep their own references.
  pub fn clear(&self) {
    self
      .entries
      .write()
      .expect("asset cache lock is never poisoned")
      .clear();
  }

  pub fn get_stats(&self) -> XrayCacheStats {
    let entries = self.entries.read().expect("asset cache lock is never poisoned");

    XrayCacheStats {
      entries: entries.len(),
      bytes: Self::retained_bytes(&entries),
      hits: self.hits.load(Ordering::Relaxed),
      misses: self.misses.load(Ordering::Relaxed),
      refused: self.refused.load(Ordering::Relaxed),
    }
  }

  fn key<T: 'static>(scope: &XrayLookupScope, path: &str) -> XrayCacheKey {
    XrayCacheKey {
      type_id: TypeId::of::<T>(),
      scope: scope.clone(),
      path: path.to_owned(),
    }
  }

  fn retained_bytes(entries: &HashMap<XrayCacheKey, XrayCacheEntry>) -> u64 {
    entries.values().map(|entry| entry.bytes).sum()
  }

  fn drop_unreferenced(entries: &mut HashMap<XrayCacheKey, XrayCacheEntry>) {
    entries.retain(|_, entry| Arc::strong_count(&entry.value) > 1);
  }
}
