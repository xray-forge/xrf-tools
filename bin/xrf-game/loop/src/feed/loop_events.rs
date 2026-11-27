use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::feed::loop_event::LoopEvent;

/// What the simulation tells presentation once (a step taken, an object gone online, a shot), in the order the ticks
/// stepped them: kept until taken, never dropped or merged however rarely presentation takes them, unlike the frames,
/// which it samples.
#[derive(Debug)]
pub struct LoopEvents<E> {
  queue: Mutex<VecDeque<LoopEvent<E>>>,
}

impl<E> Default for LoopEvents<E> {
  fn default() -> Self {
    Self {
      queue: Mutex::new(VecDeque::new()),
    }
  }
}

impl<E> LoopEvents<E> {
  /// Appends the events the step from `tick` told, in their order.
  pub fn push(&self, tick: u64, events: impl IntoIterator<Item = E>) {
    self
      .lock()
      .extend(events.into_iter().map(|event| LoopEvent { tick, event }));
  }

  /// Every event not yet taken, oldest first.
  pub fn take(&self) -> Vec<LoopEvent<E>> {
    self.lock().drain(..).collect()
  }

  fn lock(&self) -> MutexGuard<'_, VecDeque<LoopEvent<E>>> {
    self.queue.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
