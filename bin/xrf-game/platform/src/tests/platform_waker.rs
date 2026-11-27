use crate::platform_waker::PlatformWaker;

#[test]
fn a_wake_before_the_platform_takes_the_waker_calls_nothing_of_sdl() {
  let waker: PlatformWaker = PlatformWaker::default();

  // No platform runs and no library is loaded: both wakes only note themselves.
  waker.wake();
  waker.to_waker().wake();
  waker.settle();
  waker.wake();
}
