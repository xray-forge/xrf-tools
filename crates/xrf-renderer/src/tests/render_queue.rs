use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::contract::render_viewport_event::RenderViewportEvent;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::host::render_event_sink::RenderEventSink;
use crate::host::render_scene_feedback::RenderSceneFeedback;
use crate::host::render_scene_frame::RenderSceneFrame;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_view_frame::RenderViewFrame;
use crate::host::render_view_input::RenderViewInput;
use crate::host::render_window_host::RenderWindowHost;
use crate::host::render_world::RenderWorld;
use crate::tests::test_bundle::TestBundle;
use crate::tests::test_workers::create_workers;
use crate::thread::render_command::RenderCommand;
use crate::thread::render_queue::RenderQueue;
use crate::thread::render_wait::RenderWait;

/// Longer than any of these waits should take, so a broken contract fails rather than hangs.
const WAIT: Duration = Duration::from_secs(10);

/// Holds the thread at its first look at the window until the test has sent everything it asks, so the commands are
/// taken one after another, with no turn in between that would start a GPU for the attached viewport.
#[derive(Default)]
struct Gate {
  is_open: Mutex<bool>,
  opened: Condvar,
}

impl Gate {
  fn open(&self) {
    *self.is_open.lock().unwrap_or_else(PoisonError::into_inner) = true;
    self.opened.notify_all();
  }

  fn pass(&self) {
    let is_open = self.is_open.lock().unwrap_or_else(PoisonError::into_inner);
    let _ = self
      .opened
      .wait_timeout_while(is_open, WAIT, |is_open| !*is_open)
      .unwrap_or_else(PoisonError::into_inner);
  }
}

/// A minimised window, which the thread never draws into: one asked for its handles fails the test rather than hand the
/// GPU a window that is not there.
struct HiddenWindow(Arc<Gate>);

// SAFETY: it names no window, and a thread that would present through it panics in `get_handles` instead.
unsafe impl RenderWindowHost for HiddenWindow {
  fn get_key(&self) -> u64 {
    1
  }

  fn get_handles(&self) -> (RawWindowHandle, RawDisplayHandle) {
    panic!("A hidden test window has no handles to draw into");
  }

  fn get_client_size(&self) -> (u32, u32) {
    (0, 0)
  }

  fn is_minimized(&self) -> bool {
    self.0.pass();

    true
  }
}

/// A viewport's events, noting when the thread let the viewport go.
struct DroppedSink(Arc<AtomicBool>);

impl RenderEventSink for DroppedSink {
  fn send(&self, _: RenderViewportEvent) -> bool {
    true
  }
}

impl Drop for DroppedSink {
  fn drop(&mut self) {
    self.0.store(true, Ordering::Release);
  }
}

/// A world no frame is drawn of: a thread whose only viewport is hidden handles its commands without one.
struct UndrawnWorld;

impl RenderWorld for UndrawnWorld {
  fn get_asked_scene(&self, _: RenderViewportId) -> Option<RenderSceneId> {
    None
  }

  fn advance_view(&mut self, _: RenderViewportId, _: RenderViewInput<'_>) -> RenderViewFrame {
    panic!("A test thread handling commands draws no frame");
  }

  fn advance_scenes(
    &mut self,
    _: Instant,
    _: HashMap<RenderSceneId, RenderSceneFeedback>,
  ) -> HashMap<RenderSceneId, RenderSceneFrame> {
    panic!("A test thread handling commands draws no frame");
  }

  fn restart_scenes(&mut self) {}
}

fn create_queue() -> RenderQueue {
  RenderQueue::new(
    create_workers(),
    Arc::new(TestBundle::empty()),
    Arc::new(Mutex::new(UndrawnWorld)),
  )
}

/// Attaches a hidden viewport behind `gate`, answering the flag its sink raises once the thread lets it go.
fn attach(queue: &RenderQueue, id: RenderViewportId, gate: &Arc<Gate>) -> Arc<AtomicBool> {
  let dropped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

  queue.send(RenderCommand::Attach {
    id,
    host: Arc::new(HiddenWindow(Arc::clone(gate))),
    sink: Box::new(DroppedSink(Arc::clone(&dropped))),
  });

  dropped
}

#[test]
fn answers_at_once_while_no_thread_runs() {
  let queue: RenderQueue = create_queue();

  assert_eq!(queue.wait_handled(WAIT), RenderWait::NotRunning);
  assert_eq!(queue.stop(WAIT), RenderWait::NotRunning);
}

#[test]
fn answers_handled_only_once_a_detach_sent_before_is_handled() {
  let queue: RenderQueue = create_queue();
  let gate: Arc<Gate> = Arc::default();
  let id: RenderViewportId = RenderViewportId(1);
  let dropped: Arc<AtomicBool> = attach(&queue, id, &gate);
  let (reply, answer): (Sender<()>, Receiver<()>) = channel();

  queue.send(RenderCommand::Detach { id });
  queue.send(RenderCommand::Handled { reply });
  gate.open();

  assert!(answer.recv_timeout(WAIT).is_ok());
  assert!(dropped.load(Ordering::Acquire));
  // The thread still runs, with nothing attached.
  assert_eq!(queue.wait_handled(WAIT), RenderWait::Answered);
  assert_eq!(queue.stop(WAIT), RenderWait::Answered);
}

#[test]
fn stops_letting_every_viewport_go_and_leaves_no_thread_running() {
  let queue: RenderQueue = create_queue();
  let gate: Arc<Gate> = Arc::default();
  let dropped: Arc<AtomicBool> = attach(&queue, RenderViewportId(1), &gate);
  let (reply, answer): (Sender<()>, Receiver<()>) = channel();

  queue.send(RenderCommand::Stop { reply });
  gate.open();

  assert!(answer.recv_timeout(WAIT).is_ok());
  assert!(dropped.load(Ordering::Acquire));
  assert_eq!(queue.wait_handled(WAIT), RenderWait::NotRunning);
}
