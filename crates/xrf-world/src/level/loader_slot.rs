use xrf_renderer::{LoaderReceiver, RenderWorkers, take_answer};

/// One thing read once on the workers and kept: its loader's answer taken once (`take_answer`), then what it read,
/// none where it read nothing or ended without answering.
pub struct LoaderSlot<T> {
  /// The answer on its way, none once it came.
  pending: Option<LoaderReceiver<Option<T>>>,
  value: Option<T>,
  /// What it reads, named in the log should its loader end without answering.
  what: &'static str,
}

impl<T: Send + 'static> LoaderSlot<T> {
  /// Starts reading it on the workers.
  pub fn spawn(workers: &RenderWorkers, what: &'static str, read: impl FnOnce() -> Option<T> + Send + 'static) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();

    workers.spawn(move || {
      let _ = sender.send(read());
    });

    Self {
      pending: Some(receiver),
      value: None,
      what,
    }
  }

  /// What was read, taking the loader's answer once it is in; none while it is on its way or where it is not to be had.
  pub fn poll(&mut self) -> Option<&T> {
    if let Some(answer) = take_answer(&mut self.pending, self.what) {
      self.value = answer;
    }

    self.value.as_ref()
  }

  /// Whether its loader has not answered yet.
  pub fn is_pending(&self) -> bool {
    self.pending.is_some()
  }
}
