//! The recording slot's protocol over a stand-in recording, so a page going while a writer opens is tried without one.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use xrf_app_core::TauriResult;
use xrf_app_render::WorldState;
use xrf_job::testing::create_test_pool;
use xrf_renderer::{RenderViewportId, RenderWorkers};

use crate::viewer::recording::{RecordingOutcome, RecordingSlot, SlotRecording};

const VIEWPORT: RenderViewportId = RenderViewportId(1);

/// Longer than a release thread takes to finish what it took.
const WAIT: Duration = Duration::from_secs(5);

/// A recording that counts its stops.
struct FakeRecording {
  window: &'static str,
  is_ended: bool,
  stops: Arc<AtomicUsize>,
}

impl SlotRecording for FakeRecording {
  fn get_window(&self) -> &str {
    self.window
  }

  fn is_ended(&self) -> bool {
    self.is_ended
  }

  fn stop(self) -> TauriResult<RecordingOutcome> {
    self.stops.fetch_add(1, Ordering::SeqCst);

    Ok(RecordingOutcome {
      file: PathBuf::from(format!("{}.mp4", self.window)),
      duration: 0.0,
      frames: 0,
      dropped_frames: 0,
      dropped_sound: 0,
      width: 2,
      height: 2,
      encoder: String::new(),
    })
  }
}

fn world() -> WorldState {
  WorldState::new(RenderWorkers::new(Arc::new(create_test_pool(1))))
}

fn recording(window: &'static str, stops: &Arc<AtomicUsize>) -> FakeRecording {
  FakeRecording {
    window,
    is_ended: false,
    stops: Arc::clone(stops),
  }
}

/// A recording as an opener answers it, with nothing beside it.
fn opened(recording: FakeRecording) -> TauriResult<(FakeRecording, ())> {
  Ok((recording, ()))
}

/// An opener a refused start must never run.
fn never_opened() -> TauriResult<(FakeRecording, ())> {
  panic!("nothing is opened")
}

/// Waits for a release thread to have stopped `count` recordings.
fn wait_for_stops(stops: &AtomicUsize, count: usize) {
  let deadline: Instant = Instant::now() + WAIT;

  while stops.load(Ordering::SeqCst) < count {
    assert!(Instant::now() < deadline, "the released recording is stopped");
    std::thread::yield_now();
  }
}

// A start or a release holding the slot is no running recording; the start's own check under the lock decides.
#[test]
fn reads_a_slot_held_by_a_start_as_no_recording() {
  let slot: RecordingSlot<FakeRecording> = RecordingSlot::default();
  let stops: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

  slot
    .start("main", 0, &world(), VIEWPORT, || {
      assert!(!slot.is_recording(), "held while the writer opens");

      opened(recording("main", &stops))
    })
    .expect("the recording starts");

  assert!(slot.is_recording());
}

// The release runs on the event loop, which every window paints through, while a start holds the slot for a second or
// more: it counts the page gone and leaves, and the start stops what it opened for a page no longer there.
#[test]
fn stops_what_a_start_opened_for_a_page_that_went_meanwhile_without_the_release_waiting() {
  let slot: RecordingSlot<FakeRecording> = RecordingSlot::default();
  let world: WorldState = world();
  let stops: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

  let started: TauriResult<()> = slot.start("main", slot.count_releases("main"), &world, VIEWPORT, || {
    let (sender, receiver) = mpsc::channel::<()>();
    let (releasing, released_world) = (slot.clone(), world.clone());

    std::thread::spawn(move || {
      releasing.release_window("main", &released_world);
      let _ = sender.send(());
    });

    assert!(
      receiver.recv_timeout(WAIT).is_ok(),
      "the release returns while the slot is held"
    );

    opened(recording("main", &stops))
  });

  assert_eq!(
    started,
    Err(String::from(
      "The page that asked for the recording went as it started; main.mp4 was finished"
    ))
  );
  assert_eq!(stops.load(Ordering::SeqCst), 1, "what was opened is stopped");
  assert!(!slot.is_recording());
}

#[test]
fn opens_nothing_for_a_page_that_went_before_its_start_took_the_slot() {
  let slot: RecordingSlot<FakeRecording> = RecordingSlot::default();
  let world: WorldState = world();
  let released: u64 = slot.count_releases("main");

  slot.release_window("main", &world);

  assert_eq!(
    slot.start("main", released, &world, VIEWPORT, never_opened),
    Err(String::from(
      "The page that asked for the recording went before it started"
    ))
  );
}

#[test]
fn releases_a_windows_recording_and_leaves_another_windows() {
  let slot: RecordingSlot<FakeRecording> = RecordingSlot::default();
  let world: WorldState = world();
  let stops: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

  slot
    .start("main", 0, &world, VIEWPORT, || opened(recording("main", &stops)))
    .expect("the recording starts");

  slot.release_window("second", &world);
  assert!(slot.is_recording(), "another window's page going leaves it");

  slot.release_window("main", &world);
  wait_for_stops(&stops, 1);
  assert!(!slot.is_recording());
  assert_eq!(slot.count_releases("main"), 1);
  assert_eq!(slot.count_releases("second"), 1);
}

#[test]
fn refuses_a_second_start_and_collects_one_that_ended_by_itself() {
  let slot: RecordingSlot<FakeRecording> = RecordingSlot::default();
  let world: WorldState = world();
  let stops: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));

  slot
    .start("main", 0, &world, VIEWPORT, || {
      opened(FakeRecording {
        is_ended: true,
        ..recording("main", &stops)
      })
    })
    .expect("the first starts");
  slot
    .start("main", 0, &world, VIEWPORT, || opened(recording("main", &stops)))
    .expect("one that ended by itself is collected");

  assert_eq!(stops.load(Ordering::SeqCst), 1, "the ended one finished");
  assert_eq!(
    slot.start("main", 0, &world, VIEWPORT, never_opened),
    Err(String::from("A recording runs already"))
  );
  assert_eq!(
    slot.stop(&world).map(|outcome| outcome.file),
    Ok(PathBuf::from("main.mp4"))
  );
  assert_eq!(
    slot.stop(&world).map(|outcome| outcome.file),
    Err(String::from("No recording runs"))
  );
}
