use xrf_error::XrfResult;

use crate::pipeline::pipeline_cache::PipelineCache;
use crate::pipeline::pipeline_set::PipelineSet;

/// The pipelines a pass asks for only once it draws: none before its first ask makes them, then held as a `PipelineSet`
/// is, made again once the cache is invalidated past them; a refused first ask is not tried again before the cache
/// changes, so a pass whose shader does not build is not compiled again every frame.
pub struct PipelineAsk<T> {
  set: Option<PipelineSet<T>>,
  /// The cache's generation a first ask was refused in, none once one is made.
  refused: Option<u64>,
}

impl<T> Default for PipelineAsk<T> {
  fn default() -> Self {
    Self {
      set: None,
      refused: None,
    }
  }
}

impl<T> PipelineAsk<T> {
  /// Makes the pipelines with `make` where none were made yet, or again where `cache` was invalidated since.
  ///
  /// # Errors
  ///
  /// Returns `make`'s error as it refuses them, once a generation: while a refused first ask stands, the next answer
  /// `Ok` with nothing made, the refusal already told, until the cache changes; a refused remake keeps the last made.
  pub fn ask(&mut self, cache: &PipelineCache, make: impl FnOnce() -> XrfResult<T>) -> XrfResult {
    if let Some(set) = &mut self.set {
      return set.refresh(cache, make);
    }

    let generation: u64 = cache.get_generation();

    if self.refused == Some(generation) {
      return Ok(());
    }

    match make() {
      Ok(pipelines) => {
        self.set = Some(PipelineSet::new(cache, pipelines));
        self.refused = None;

        Ok(())
      }
      Err(error) => {
        self.refused = Some(generation);

        Err(error)
      }
    }
  }

  /// The pipelines made, none before an ask made them.
  pub fn get(&self) -> Option<&T> {
    self.set.as_deref()
  }
}
