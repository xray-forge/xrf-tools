use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::task::{Wake, Waker};

use sdl3::EventSubsystem;
use sdl3::event::{Event, EventSender, EventType};

/// How another thread ends the main thread's wait for the platform's events (`Platform::wait`), so the main thread
/// rests until there is input, its own next work is due, or a thread has something for it. It may be made before the
/// platform starts and handed to threads that start first: until the platform takes it (`Platform::attach`) a wake
/// only notes itself, and the platform's first wait after taking it returns at once. Wakes asked while one's event
/// still waits in the platform's queue count as that one.
#[derive(Clone, Default)]
pub struct PlatformWaker {
  state: Arc<PlatformWakerState>,
}

/// Pushes the event that ends the platform's wait, answering why it could not.
type PlatformWakePush = Box<dyn Fn() -> Result<(), String> + Send + Sync>;

/// What a waker's handles share. A waking thread writes one flag and reads another while the main thread does the
/// reverse (a wake against the platform taking the waker, a wake against its event being taken), so every access is
/// ordered `SeqCst`: under a weaker order both could read the other's old value and a wake be lost.
#[derive(Default)]
struct PlatformWakerState {
  /// How a wake's event is pushed to the platform's queue, once the platform took the waker.
  push: OnceLock<PlatformWakePush>,
  /// Whether the platform runs, so its events may be pushed to.
  is_attached: AtomicBool,
  /// Whether a wake's event waits in the platform's queue, or a wake waits for the platform to take the waker. Set
  /// by the wake that pushes and cleared only as that event is taken, so it never says an event waits when none does:
  /// a wake skipped for it is always answered by the event still to be taken.
  is_pending: AtomicBool,
}

impl PlatformWakerState {
  /// Pushes the event that ends the wait where the platform runs. One that cannot be pushed leaves nothing pending,
  /// so the next wake tries again.
  fn push(&self) {
    let Some(push) = self.push.get().filter(|_| self.is_attached.load(Ordering::SeqCst)) else {
      return;
    };

    if let Err(error) = push() {
      log::warn!("The main thread was not woken: {error}");
      self.is_pending.store(false, Ordering::SeqCst);
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

  /// Lets wakes reach the platform's `events` from now, with one for a wake asked for before; SDL takes an event
  /// pushed from any thread.
  pub(crate) fn attach(&self, events: &EventSubsystem) {
    let sender: EventSender = events.event_sender();

    self.attach_push(move || {
      // SDL's first user event number, which nothing else in the game pushes: the wait only has to return on it.
      sender
        .push_event(Event::User {
          timestamp: 0,
          window_id: 0,
          type_: EventType::User as u32,
          code: 0,
          data1: std::ptr::null_mut(),
          data2: std::ptr::null_mut(),
        })
        .map_err(|error| error.to_string())
    });
  }

  /// Lets wakes reach the platform through `push` from now, with one for a wake asked for before.
  pub(crate) fn attach_push(&self, push: impl Fn() -> Result<(), String> + Send + Sync + 'static) {
    // A waker taken twice keeps its first way to push, which reaches the same queue.
    let _ = self.state.push.set(Box::new(push));

    self.state.is_attached.store(true, Ordering::SeqCst);

    if self.state.is_pending.load(Ordering::SeqCst) {
      self.state.push();
    }
  }

  /// Keeps wakes from the platform's events, which are closing.
  pub(crate) fn detach(&self) {
    self.state.is_attached.store(false, Ordering::SeqCst);
  }

  /// Notes that the wake's event was taken from the platform's queue: a thread waking from now pushes another. The
  /// platform calls it for each such event it takes, before its caller looks at what the threads brought, so what a
  /// thread brought before a wake that was skipped is still looked at.
  pub(crate) fn settle(&self) {
    self.state.is_pending.store(false, Ordering::SeqCst);
  }
}
