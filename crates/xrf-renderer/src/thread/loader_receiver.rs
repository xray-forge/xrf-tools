use std::sync::mpsc::{Receiver, Sender, TryIter, TryRecvError, channel};
use std::sync::{Mutex, PoisonError};

/// The end of a worker's channel the renderer's state holds. That state is shared by the passes recording a frame, so
/// it must be `Sync`, which a bare `Receiver` is not; this one is read through `&mut self` alone, so its lock is never
/// contended.
pub struct LoaderReceiver<T> {
  receiver: Mutex<Receiver<T>>,
}

impl<T> LoaderReceiver<T> {
  /// A channel whose receiving end is a `LoaderReceiver`.
  pub fn channel() -> (Sender<T>, Self) {
    let (sender, receiver) = channel();

    (
      sender,
      Self {
        receiver: Mutex::new(receiver),
      },
    )
  }

  pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
    self
      .receiver
      .get_mut()
      .unwrap_or_else(PoisonError::into_inner)
      .try_recv()
  }

  /// Every answer already sent, without waiting.
  pub fn try_iter(&mut self) -> TryIter<'_, T> {
    self
      .receiver
      .get_mut()
      .unwrap_or_else(PoisonError::into_inner)
      .try_iter()
  }
}
