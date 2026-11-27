use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::task::{Wake, Waker};

use sdl3::EventSubsystem;
use sdl3::event::{Event, EventSender, EventType};

/// How another thread ends the main thread's wait for the platform's events (`Platform::wait`), so the main thread
/// rests until there is input, its own next work is due, or a thread has something for it. It may be made before the
/// platform starts and handed to threads that start first: until the platform takes it (`Platform::attach`) a wake
/// only notes itself, and the platform's first wait after taking it returns at once. Wakes between two waits count as
/// one.
#[derive(Clone, Default)]
pub struct PlatformWaker {
  state: Arc<PlatformWakerState>,
}

/// What a waker's handles share. A waking thread writes one flag and reads another while the main thread does the
/// reverse (a wake against the platform taking the waker, a wake against a wait ending), so every access is ordered
/// `SeqCst`: under a weaker order both could read the other's old value and a wake be lost.
#[derive(Default)]
struct PlatformWakerState {
  /// Where the platform's events are pushed to, once it took the waker.
  sender: OnceLock<EventSender>,
  /// Whether the platform runs, so its events may be pushed to.
  is_attached: AtomicBool,
  /// Whether a wake was asked for since the last wait ended.
  is_pending: AtomicBool,
}

impl PlatformWakerState {
  /// Pushes the event that ends the wait where the platform runs; SDL takes one from any thread.
  fn push(&self) {
    let Some(sender) = self.sender.get().filter(|_| self.is_attached.load(Ordering::SeqCst)) else {
      return;
    };
    // SDL's first user event number, which nothing else in the game pushes: the wait only has to return on it.
    let event: Event = Event::User {
      timestamp: 0,
      window_id: 0,
      type_: EventType::User as u32,
      code: 0,
      data1: std::ptr::null_mut(),
      data2: std::ptr::null_mut(),
    };

    if let Err(error) = sender.push_event(event) {
      log::warn!("The main thread was not woken: {error}");
    }
  }
}

impl Wake for PlatformWakerState {
  fn wake(self: Arc<Self>) {
    self.wake_by_ref();
  }

  fn wake_by_ref(self: &Arc<Self>) {
    if !self.is_pending.swap(true, Ordering::SeqCst) {
      self.push();
    }
  }
}

impl PlatformWaker {
  /// Ends the main thread's wait, or its next one where it is not waiting.
  pub fn wake(&self) {
    self.state.wake_by_ref();
  }

  /// The same waker as the standard library's, for a crate that wakes the main thread and knows nothing of the
  /// platform.
  pub fn to_waker(&self) -> Waker {
    Waker::from(Arc::clone(&self.state))
  }

  /// Lets wakes reach the platform's `events` from now, with one for a wake asked for before.
  pub(crate) fn attach(&self, events: &EventSubsystem) {
    // A waker taken twice keeps its first sender, which pushes to the same queue.
    let _ = self.state.sender.set(events.event_sender());

    self.state.is_attached.store(true, Ordering::SeqCst);

    if self.state.is_pending.load(Ordering::SeqCst) {
      self.state.push();
    }
  }

  /// Keeps wakes from the platform's events, which are closing.
  pub(crate) fn detach(&self) {
    self.state.is_attached.store(false, Ordering::SeqCst);
  }

  /// Notes that a wait ended: what a thread brings from now asks for a wake of its own.
  pub(crate) fn settle(&self) {
    self.state.is_pending.store(false, Ordering::SeqCst);
  }
}
