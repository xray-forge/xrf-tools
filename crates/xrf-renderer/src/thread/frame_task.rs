use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, PoisonError};

use rayon::ThreadPool;

/// The work a task runs, told whether it runs on a worker and may spread over the pool.
type FrameTaskWork<T> = Box<dyn FnOnce(bool) -> T + Send>;

/// Work a frame waits for, run by whichever reaches it first: a worker free to take it, or the thread joining it, so
/// the loaders queued ahead of it on the pool cannot hold the frame.
pub(crate) struct FrameTask<T> {
  work: Arc<Mutex<Option<FrameTaskWork<T>>>>,
  receiver: Receiver<T>,
}

impl<T: Send + 'static> FrameTask<T> {
  /// Offers `work` to the pool: a worker taking it runs it there, told it may spread over the pool.
  pub(crate) fn start(pool: &ThreadPool, work: impl FnOnce(bool) -> T + Send + 'static) -> Self {
    let work: Arc<Mutex<Option<FrameTaskWork<T>>>> = Arc::new(Mutex::new(Some(Box::new(work))));
    let offered: Arc<Mutex<Option<FrameTaskWork<T>>>> = Arc::clone(&work);
    let (sender, receiver) = mpsc::channel();

    pool.spawn(move || {
      if let Some(work) = Self::claim(&offered) {
        let _ = sender.send(work(true));
      }
    });

    Self { work, receiver }
  }

  /// Its answer: run here, told it may not spread over the pool, when no worker took it yet, else waited for. None
  /// where a worker took it and ended without answering.
  pub(crate) fn join(self) -> Option<T> {
    match Self::claim(&self.work) {
      Some(work) => Some(work(false)),
      None => self.receiver.recv().ok(),
    }
  }

  /// The work, for whichever asks first.
  fn claim(work: &Mutex<Option<FrameTaskWork<T>>>) -> Option<FrameTaskWork<T>> {
    work.lock().unwrap_or_else(PoisonError::into_inner).take()
  }
}

impl<T> Drop for FrameTask<T> {
  /// Lets go of work no worker took yet, with what it holds, so a task no one joins runs nowhere.
  fn drop(&mut self) {
    drop(self.work.lock().unwrap_or_else(PoisonError::into_inner).take());
  }
}
