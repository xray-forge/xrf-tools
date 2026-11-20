use std::ops::Deref;

use xrf_error::XrfResult;

use crate::pipeline::pipeline_cache::PipelineCache;

/// The pipelines a pass draws with and the cache's generation they were made in: made again once the cache is
/// invalidated past it (a shader reload), the last made kept where the cache refuses the remake, which is not tried again
/// before the cache changes. `T` is what the pass draws with: one pipeline, an array of them, or a struct naming each;
/// it reads them through `Deref`.
pub struct PipelineSet<T> {
  pipelines: T,
  generation: u64,
}

impl<T> PipelineSet<T> {
  /// `pipelines`, made from `cache` as it stands.
  pub fn new(cache: &PipelineCache, pipelines: T) -> Self {
    Self {
      pipelines,
      generation: cache.get_generation(),
    }
  }

  /// Makes the pipelines again with `make` where `cache` was invalidated since they were made.
  ///
  /// # Errors
  ///
  /// Returns `make`'s error, the last made kept.
  pub fn refresh(&mut self, cache: &PipelineCache, make: impl FnOnce() -> XrfResult<T>) -> XrfResult {
    let generation: u64 = cache.get_generation();

    if generation == self.generation {
      return Ok(());
    }

    self.generation = generation;
    self.pipelines = make()?;

    Ok(())
  }
}

impl<T> Deref for PipelineSet<T> {
  type Target = T;

  fn deref(&self) -> &T {
    &self.pipelines
  }
}
