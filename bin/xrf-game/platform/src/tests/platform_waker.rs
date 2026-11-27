use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::platform_waker::PlatformWaker;

/// A waker whose wakes reach a count of the events pushed, in place of the platform's queue.
struct Counted {
  waker: PlatformWaker,
  pushed: Arc<AtomicU32>,
}

impl Counted {
  /// A waker a platform took, every push succeeding.
  fn attach() -> Self {
    let counted: Self = Self {
      waker: PlatformWaker::default(),
      pushed: Arc::new(AtomicU32::new(0)),
    };
    let pushed: Arc<AtomicU32> = Arc::clone(&counted.pushed);

    counted.waker.attach_push(move || {
      pushed.fetch_add(1, Ordering::AcqRel);

      Ok(())
    });

    counted
  }

  fn count(&self) -> u32 {
    self.pushed.load(Ordering::Acquire)
  }
}

#[test]
fn a_wake_before_the_platform_takes_the_waker_calls_nothing_of_sdl() {
  let waker: PlatformWaker = PlatformWaker::default();

  // No platform runs and no library is loaded: both wakes only note themselves.
  waker.wake();
  waker.to_waker().wake();
  waker.settle();
  waker.wake();
}

#[test]
fn pushes_one_event_for_the_wakes_asked_before_the_platform_took_it() {
  let waker: PlatformWaker = PlatformWaker::default();
  let pushed: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
  let counted: Arc<AtomicU32> = Arc::clone(&pushed);

  waker.wake();
  waker.to_waker().wake();
  assert_eq!(pushed.load(Ordering::Acquire), 0);

  waker.attach_push(move || {
    counted.fetch_add(1, Ordering::AcqRel);

    Ok(())
  });

  assert_eq!(pushed.load(Ordering::Acquire), 1);
}

#[test]
fn counts_the_wakes_asked_while_its_event_waits_as_one_and_pushes_again_once_it_is_taken() {
  let counted: Counted = Counted::attach();

  counted.waker.wake();
  counted.waker.to_waker().wake();
  counted.waker.wake();
  assert_eq!(counted.count(), 1);

  // The platform took the event.
  counted.waker.settle();
  counted.waker.wake();
  assert_eq!(counted.count(), 2);
}

#[test]
fn pushes_again_after_an_event_pushed_and_taken_within_the_pass_that_took_the_one_before() {
  let counted: Counted = Counted::attach();

  // One pass of the platform over its queue: it takes a wake's event, a thread wakes meanwhile and its event is
  // taken by the same pass.
  counted.waker.wake();
  counted.waker.settle();
  counted.waker.wake();
  counted.waker.settle();
  assert_eq!(counted.count(), 2);

  // Nothing waits in the queue now, so the next wake must push, or the main thread would rest through it.
  counted.waker.wake();
  assert_eq!(counted.count(), 3);
}

#[test]
fn tries_again_at_the_next_wake_after_an_event_that_could_not_be_pushed() {
  let waker: PlatformWaker = PlatformWaker::default();
  let tried: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
  let counted: Arc<AtomicU32> = Arc::clone(&tried);

  waker.attach_push(move || {
    // The first push fails, as a full queue refuses one.
    if counted.fetch_add(1, Ordering::AcqRel) == 0 {
      return Err(String::from("the queue is full"));
    }

    Ok(())
  });

  waker.wake();
  waker.wake();
  assert_eq!(tried.load(Ordering::Acquire), 2);

  // The second was pushed and waits: a third wake counts as it.
  waker.wake();
  assert_eq!(tried.load(Ordering::Acquire), 2);
}

#[test]
fn pushes_nothing_once_the_platform_let_it_go() {
  let counted: Counted = Counted::attach();

  counted.waker.detach();
  counted.waker.wake();
  assert_eq!(counted.count(), 0);
}
