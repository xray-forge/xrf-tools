use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;

use xrf_error::{XrfError, XrfResult};

use crate::pipeline::pipeline_cache::PipelineCache;

/// Pipelines a pass makes one set a key on first use, such as a window's format, and the cache's generation they were
/// made in: all made again once the cache is invalidated past it, every last made kept where the cache refuses one
/// remake, which is not tried again before the cache changes; a key refused on first use is not made again either
/// before then.
pub struct PipelineMap<K, T> {
  pipelines: HashMap<K, T>,
  /// Why each key refused on first use, kept until the cache changes.
  refused: HashMap<K, String>,
  generation: u64,
}

impl<K: Copy + Eq + Hash, T> PipelineMap<K, T> {
  /// None made yet, from `cache` as it stands.
  pub fn new(cache: &PipelineCache) -> Self {
    Self {
      pipelines: HashMap::new(),
      refused: HashMap::new(),
      generation: cache.get_generation(),
    }
  }

  /// Makes the pipelines for `key` with `make`, unless they are made or were refused since the cache last changed.
  ///
  /// # Errors
  ///
  /// Returns `make`'s refusal, nothing kept for `key`, and the same refusal word for word at every ask after it until
  /// the cache changes, so a log that skips a repeated failure tells it once.
  pub fn prepare(&mut self, key: K, make: impl FnOnce(K) -> XrfResult<T>) -> XrfResult {
    if let Entry::Vacant(entry) = self.pipelines.entry(key)
      && !self.refused.contains_key(&key)
    {
      match make(key) {
        Ok(pipelines) => {
          entry.insert(pipelines);
        }
        Err(error) => {
          self.refused.insert(key, error.to_string());
        }
      }
    }

    match self.refused.get(&key) {
      Some(refusal) => Err(XrfError::new_unexpected_error(refusal.clone())),
      None => Ok(()),
    }
  }

  /// The pipelines made for `key`, none before `prepare` made them.
  pub fn get(&self, key: &K) -> Option<&T> {
    self.pipelines.get(key)
  }

  /// Makes every key's pipelines again with `make` where `cache` was invalidated since they were made, and lets a key
  /// refused before be asked again.
  ///
  /// # Errors
  ///
  /// Returns the first of `make`'s errors, every last made kept.
  pub fn refresh(&mut self, cache: &PipelineCache, mut make: impl FnMut(K) -> XrfResult<T>) -> XrfResult {
    let generation: u64 = cache.get_generation();

    if generation == self.generation {
      return Ok(());
    }

    self.generation = generation;
    self.refused.clear();
    self.pipelines = self
      .pipelines
      .keys()
      .map(|key| Ok((*key, make(*key)?)))
      .collect::<XrfResult<HashMap<K, T>>>()?;

    Ok(())
  }
}
