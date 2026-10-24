use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use xrf_app_core::types::TauriResult;

use crate::render_answer::await_answer;

fn wait(answer: Receiver<u32>) -> TauriResult<u32> {
  tauri::async_runtime::block_on(await_answer(
    answer,
    Duration::from_millis(10),
    String::from("Viewport 3 answered no frame"),
  ))
}

// A reply the render thread let go unanswered is told apart from one that never came in time.
#[test]
fn tells_a_reply_let_go_apart_from_one_that_never_came() {
  let (reply, answer): (Sender<u32>, Receiver<u32>) = channel();

  drop(reply);

  assert_eq!(
    wait(answer),
    Err(String::from(
      "Viewport 3 answered no frame: the renderer does not draw it"
    ))
  );

  let (_reply, answer): (Sender<u32>, Receiver<u32>) = channel();

  assert_eq!(wait(answer), Err(String::from("Viewport 3 answered no frame")));
}
