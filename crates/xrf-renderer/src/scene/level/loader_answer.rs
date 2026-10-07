use std::sync::mpsc::TryRecvError;

use crate::thread::loader_receiver::LoaderReceiver;

/// Takes what a loader thread answered, once, leaving nothing pending afterwards; a loader gone without answering, as
/// one that panicked is, also ends the wait, so the level still settles as loaded.
pub fn take_answer<T>(pending: &mut Option<LoaderReceiver<T>>, what: &str) -> Option<T> {
  let answer: Result<T, TryRecvError> = pending.as_mut()?.try_recv();

  match answer {
    Ok(value) => {
      *pending = None;

      Some(value)
    }
    Err(TryRecvError::Empty) => None,
    Err(TryRecvError::Disconnected) => {
      *pending = None;
      log::error!("The level's {what} loader stopped without answering");

      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::take_answer;
  use crate::thread::loader_receiver::LoaderReceiver;

  #[test]
  fn takes_an_answer_once_and_waits_for_none_after() {
    let (sender, receiver) = LoaderReceiver::channel();
    let mut pending: Option<LoaderReceiver<u32>> = Some(receiver);

    assert_eq!(take_answer(&mut pending, "test"), None);
    assert!(pending.is_some());

    sender.send(7).unwrap();

    assert_eq!(take_answer(&mut pending, "test"), Some(7));
    assert!(pending.is_none());
  }

  #[test]
  fn ends_the_wait_for_a_loader_gone_without_answering() {
    let (sender, receiver) = LoaderReceiver::<u32>::channel();
    let mut pending: Option<LoaderReceiver<u32>> = Some(receiver);

    drop(sender);

    assert_eq!(take_answer(&mut pending, "test"), None);
    assert!(pending.is_none());
  }
}
