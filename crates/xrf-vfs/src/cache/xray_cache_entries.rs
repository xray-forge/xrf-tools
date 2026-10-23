use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::XrayFileStamp;
use crate::cache::xray_cache_entry::XrayCacheEntry;
use crate::cache::xray_cache_key::XrayCacheKey;

/// The retained values with their running size, so an insert never sums the store.
#[derive(Debug, Default)]
pub(crate) struct XrayCacheEntries {
  pub(crate) map: HashMap<XrayCacheKey, XrayCacheEntry>,
  pub(crate) bytes: u64,
  pub(crate) retained: u64,
}

impl XrayCacheEntries {
  pub(crate) fn insert(
    &mut self,
    key: XrayCacheKey,
    value: Arc<dyn Any + Send + Sync>,
    bytes: u64,
    stamp: Option<XrayFileStamp>,
  ) {
    self.retained += 1;
    self.bytes += bytes;

    let order: u64 = self.retained;

    if let Some(replaced) = self.map.insert(
      key,
      XrayCacheEntry {
        value,
        bytes,
        stamp,
        order,
      },
    ) {
      self.bytes -= replaced.bytes;
    }
  }

  /// Keeps the entries `keep` admits, answering how many went.
  pub(crate) fn retain(&mut self, mut keep: impl FnMut(&XrayCacheKey, &XrayCacheEntry) -> bool) -> usize {
    let before: usize = self.map.len();
    let mut dropped: u64 = 0;

    self.map.retain(|key, entry| {
      let is_kept: bool = keep(key, entry);

      dropped += if is_kept { 0 } else { entry.bytes };

      is_kept
    });

    self.bytes -= dropped;

    before - self.map.len()
  }

  /// Drops the oldest values nobody holds until `bytes` more fit under `budget`, or none such is left.
  pub(crate) fn evict_for(&mut self, bytes: u64, budget: u64) {
    let mut unheld: Vec<(u64, XrayCacheKey)> = self
      .map
      .iter()
      .filter(|(_, entry)| Arc::strong_count(&entry.value) == 1)
      .map(|(key, entry)| (entry.order, key.clone()))
      .collect();

    unheld.sort_unstable_by_key(|(order, _)| *order);

    for (_, key) in unheld {
      if self.bytes + bytes <= budget {
        break;
      }

      if let Some(entry) = self.map.remove(&key) {
        self.bytes -= entry.bytes;
      }
    }
  }
}
