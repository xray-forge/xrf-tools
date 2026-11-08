use xrf_renderer::RenderWorkers;

use crate::level::loading::loader_once::LoaderOnce;

/// One thing read once on the workers and kept: its loader's answer taken once (`LoaderOnce`), then what it read,
/// none where it read nothing or ended without answering.
pub struct LoaderSlot<T> {
  loader: LoaderOnce<T>,
  value: Option<T>,
}

impl<T: Send + 'static> LoaderSlot<T> {
  /// Starts reading it on the workers.
  pub fn spawn(workers: &RenderWorkers, what: &'static str, read: impl FnOnce() -> Option<T> + Send + 'static) -> Self {
    Self {
      loader: LoaderOnce::spawn(workers, what, read),
      value: None,
    }
  }

  /// What was read, taking the loader's answer once it is in; none while it is on its way or where it is not to be had.
  pub fn poll(&mut self) -> Option<&T> {
    if let Some(value) = self.loader.take() {
      self.value = Some(value);
    }

    self.value.as_ref()
  }

  /// Whether its loader has not answered yet.
  #[cfg(test)]
  pub fn is_pending(&self) -> bool {
    self.loader.is_pending()
  }
}
