use xrf_renderer::{LoaderReceiver, RenderWorkers, take_answer};

/// One thing read once on the workers and taken once: what its loader answered, none while it is on its way, once
/// taken, where it read nothing, or where the loader ended without answering.
pub(crate) struct LoaderOnce<T> {
  /// The answer on its way, none once it came.
  pending: Option<LoaderReceiver<Option<T>>>,
  /// What it reads, named in the log should its loader end without answering.
  what: &'static str,
}

impl<T: Send + 'static> LoaderOnce<T> {
  /// Starts reading it on the workers; `read` answers none where there is nothing to read, a failure logged by it.
  pub(crate) fn spawn(
    workers: &RenderWorkers,
    what: &'static str,
    read: impl FnOnce() -> Option<T> + Send + 'static,
  ) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();

    workers.spawn(move || {
      let _ = sender.send(read());
    });

    Self {
      pending: Some(receiver),
      what,
    }
  }

  /// What was read, once its loader answered; none before, and none after it was taken.
  pub(crate) fn take(&mut self) -> Option<T> {
    take_answer(&mut self.pending, self.what).flatten()
  }

  /// Whether its loader has not answered yet.
  pub(crate) fn is_pending(&self) -> bool {
    self.pending.is_some()
  }
}
