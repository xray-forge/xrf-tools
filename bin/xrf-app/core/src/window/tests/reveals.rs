use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use crate::window::WindowReveals;
use crate::window::held_window::HeldWindow;

/// A window that reports each reveal instead of showing.
struct RecordedWindow(Sender<()>);

impl HeldWindow for RecordedWindow {
  fn reveal(&self) {
    self.0.send(()).expect("the test to be listening");
  }
}

fn hold(reveals: &WindowReveals, label: &str, timeout: Duration) -> Receiver<()> {
  let (sender, receiver) = channel();

  reveals.hold_for(label.to_owned(), Box::new(RecordedWindow(sender)), timeout);

  receiver
}

#[test]
fn a_page_reveals_its_window_once() {
  let reveals: WindowReveals = WindowReveals::default();
  let revealed: Receiver<()> = hold(&reveals, "main", Duration::from_secs(60));

  assert!(!reveals.reveal("other"));
  assert!(reveals.reveal("main"));
  assert!(!reveals.reveal("main"));

  assert_eq!(revealed.try_recv(), Ok(()));
  assert!(revealed.try_recv().is_err());
}

#[test]
fn a_window_its_page_never_reveals_shows_once_the_timeout_passes() {
  let reveals: WindowReveals = WindowReveals::default();
  let revealed: Receiver<()> = hold(&reveals, "main", Duration::from_millis(10));

  assert_eq!(revealed.recv_timeout(Duration::from_secs(5)), Ok(()));
  assert!(!reveals.reveal("main"));
}
