use std::sync::mpsc::TryRecvError;

use xrf_renderer::{LoaderReceiver, RenderWorkers};

/// One thing read once on the workers: on its way, read, or not to be had, as a loader that answered nothing or ended
/// without answering leaves it.
pub enum LoaderSlot<T> {
  Pending(LoaderReceiver<Option<T>>),
  Ready(T),
  Missing,
}

impl<T: Send + 'static> LoaderSlot<T> {
  /// Starts reading it on the workers.
  pub fn spawn(workers: &RenderWorkers, read: impl FnOnce() -> Option<T> + Send + 'static) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();

    workers.spawn(move || {
      let _ = sender.send(read());
    });

    Self::Pending(receiver)
  }

  /// What was read, taking the loader's answer once it is in; none while it is on its way or where it is not to be had.
  pub fn poll(&mut self) -> Option<&T> {
    if let Self::Pending(receiver) = self {
      *self = match receiver.try_recv() {
        Ok(Some(value)) => Self::Ready(value),
        Ok(None) | Err(TryRecvError::Disconnected) => Self::Missing,
        Err(TryRecvError::Empty) => return None,
      };
    }

    match self {
      Self::Ready(value) => Some(value),
      Self::Pending(_) | Self::Missing => None,
    }
  }

  /// Whether its loader has not answered yet.
  pub fn is_pending(&self) -> bool {
    matches!(self, Self::Pending(_))
  }
}
