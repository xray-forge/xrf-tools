use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::task::{Wake, Waker};
use std::time::{Duration, Instant};

use crate::clock::loop_clock::LoopClock;
use crate::feed::loop_status::LoopStatus;
use crate::loop_stepper::LoopStepper;
use crate::record::loop_command_log::LoopCommandLog;
use crate::record::loop_session::LoopSession;
use crate::record::loop_witness::LoopWitness;
use crate::tests::test_state::{TestCommand, TestFrame, TestState};
use crate::thread::game_loop::GameLoop;

/// Longer than any of these waits should take, so a broken contract fails rather than hangs.
const WAIT: Duration = Duration::from_secs(5);

fn start() -> GameLoop<TestState> {
  start_session(LoopSession::default())
}

fn start_session(session: LoopSession<TestState>) -> GameLoop<TestState> {
  let clock: LoopClock = LoopClock::new(Instant::now(), LoopClock::DEFAULT_TICK_RATE);
  let stepper: LoopStepper<TestState> = LoopStepper::new(clock.get_step(), TestState::default());

  GameLoop::start(stepper, clock, session).expect("the thread starts")
}

/// What a witness saw of a session: the commands each tick took, and the value every step left, the first at tick 1.
#[derive(Clone, Debug, Default, PartialEq)]
struct Witnessed {
  log: LoopCommandLog<TestCommand>,
  values: Vec<f32>,
}

/// A witness keeping what it sees where the test reads it.
struct TestWitness(Arc<Mutex<Witnessed>>);

impl LoopWitness<TestState> for TestWitness {
  fn see_commands(&mut self, tick: u64, commands: &[TestCommand]) {
    let mut witnessed = self.0.lock().expect("the witness's lock");

    for command in commands {
      witnessed.log.push(tick, *command).expect("ticks in order");
    }
  }

  fn see_step(&mut self, stepper: &LoopStepper<TestState>) {
    let mut witnessed = self.0.lock().expect("the witness's lock");

    witnessed.values.push(stepper.get_state().counter.value);
    witnessed.log.end_at(stepper.get_time().tick);
  }
}

/// A session watched by a witness, and what the witness keeps of it.
struct Watched {
  thread: GameLoop<TestState>,
  witnessed: Arc<Mutex<Witnessed>>,
}

impl Watched {
  /// Starts a watched session, live or replaying `replay`.
  fn start(replay: Option<LoopCommandLog<TestCommand>>) -> Self {
    let witnessed: Arc<Mutex<Witnessed>> = Arc::new(Mutex::new(Witnessed::default()));
    let thread: GameLoop<TestState> = start_session(LoopSession {
      replay,
      witness: Some(Box::new(TestWitness(Arc::clone(&witnessed)))),
      waker: None,
    });

    Self { thread, witnessed }
  }

  fn send(&self, command: TestCommand) {
    self
      .thread
      .get_sender()
      .send(command)
      .expect("the thread takes commands");
  }

  fn read(&self) -> Witnessed {
    self.witnessed.lock().expect("the witness's lock").clone()
  }
}

/// The tick the loop stands at, read between two of its ticks.
fn read_tick(thread: &GameLoop<TestState>) -> u64 {
  let answer: Receiver<u64> = thread.snapshot(|stepper| stepper.get_time().tick);

  answer.recv_timeout(WAIT).expect("the loop answers a snapshot")
}

/// Waits until the loop passed `tick`.
fn wait_past(thread: &GameLoop<TestState>, tick: u64) {
  let deadline: Instant = Instant::now() + WAIT;

  while read_tick(thread) <= tick {
    assert!(Instant::now() < deadline, "the loop never passed tick {tick}");
    std::thread::sleep(Duration::from_millis(1));
  }
}

/// Counts the wakes it is asked for.
#[derive(Default)]
struct CountingWake(AtomicU64);

impl Wake for CountingWake {
  fn wake(self: Arc<Self>) {
    self.wake_by_ref();
  }

  fn wake_by_ref(self: &Arc<Self>) {
    self.0.fetch_add(1, Ordering::AcqRel);
  }
}

#[test]
fn wakes_its_reader_to_each_published_frame_and_to_its_stop() {
  let wakes: Arc<CountingWake> = Arc::new(CountingWake::default());
  let thread: GameLoop<TestState> = start_session(LoopSession {
    waker: Some(Waker::from(Arc::clone(&wakes))),
    ..LoopSession::default()
  });

  wait_past(&thread, 3);

  let feed = Arc::clone(thread.get_feed());

  drop(thread);

  // The frame published at the start wakes nobody, as its reader is only then handed the loop; each tick's does, and
  // the stop adds one.
  let stepped: u64 = feed.get_latest().expect("a frame").tick;

  assert_eq!(feed.get_status(), LoopStatus::Stopped);
  assert_eq!(wakes.0.load(Ordering::Acquire), stepped + 1);
}

#[test]
fn steps_on_its_thread_and_takes_commands_until_dropped() {
  let thread: GameLoop<TestState> = start();

  assert_eq!(thread.get_feed().get_latest().map(|frame| frame.tick), Some(0));

  thread
    .get_sender()
    .send(TestCommand::Add(7.0))
    .expect("the thread takes commands");

  let deadline: Instant = Instant::now() + WAIT;

  while thread.get_feed().get_latest().map(|frame| frame.state) != Some(TestFrame(7.0)) {
    assert!(Instant::now() < deadline, "the command never reached a published frame");
    std::thread::sleep(Duration::from_millis(1));
  }

  let sender = thread.get_sender();
  let feed = Arc::clone(thread.get_feed());

  drop(thread);

  // Stopped, it takes no more, and says it stopped.
  assert!(sender.send(TestCommand::Add(1.0)).is_err());
  assert_eq!(feed.get_status(), LoopStatus::Stopped);
}

#[test]
fn queues_every_steps_events_in_tick_order() {
  let thread: GameLoop<TestState> = start();

  thread
    .get_sender()
    .send(TestCommand::SetSpeed(1.0))
    .expect("the thread takes commands");
  wait_past(&thread, 20);

  let events = thread.get_events().take();
  let ticks: Vec<u64> = events.iter().map(|event| event.tick).collect();

  // One a step, none skipped however rarely they are taken: the value rises by one each tick after the speed is set.
  assert!(ticks.len() >= 20);
  assert!(ticks.windows(2).all(|pair| pair[1] == pair[0] + 1), "{ticks:?}");
  assert!(events.windows(2).all(|pair| pair[1].event - pair[0].event <= 1.0));
}

#[test]
fn stops_on_a_steps_fault_and_says_why() {
  let thread: GameLoop<TestState> = start();

  wait_past(&thread, 2);
  thread
    .get_sender()
    .send(TestCommand::Fail)
    .expect("the thread takes commands");

  let deadline: Instant = Instant::now() + WAIT;

  let status: LoopStatus = loop {
    let status: LoopStatus = thread.get_feed().get_status();

    if status != LoopStatus::Running {
      break status;
    }

    assert!(Instant::now() < deadline, "the loop never faulted");
    std::thread::sleep(Duration::from_millis(1));
  };

  assert!(
    matches!(&status, LoopStatus::Faulted { message, .. } if message.contains("told to fail")),
    "{status:?}"
  );
  // It steps no further, so no command reaches it.
  assert!(
    thread
      .snapshot(|stepper| stepper.get_time().tick)
      .recv_timeout(WAIT)
      .is_err()
  );
}

#[test]
fn holds_its_ticks_and_commands_while_paused_and_answers_snapshots_meanwhile() {
  let thread: GameLoop<TestState> = start();

  wait_past(&thread, 2);
  thread.pause();

  // The pause is taken at the next tick's start; from then the tick stands.
  let paused: u64 = {
    let deadline: Instant = Instant::now() + WAIT;
    let mut last: u64 = read_tick(&thread);

    loop {
      std::thread::sleep(Duration::from_millis(30));

      let now: u64 = read_tick(&thread);

      if now == last {
        break now;
      }

      assert!(Instant::now() < deadline, "the loop never paused");
      last = now;
    }
  };

  thread
    .get_sender()
    .send(TestCommand::Add(3.0))
    .expect("the thread takes commands while paused");
  std::thread::sleep(Duration::from_millis(30));

  let value: f32 = thread
    .snapshot(|stepper| stepper.get_state().counter.value)
    .recv_timeout(WAIT)
    .expect("a paused loop answers a snapshot");

  assert_eq!(read_tick(&thread), paused);
  assert_eq!(value, 0.0, "a command waits for the loop to step again");

  thread.resume();
  wait_past(&thread, paused);

  let value: f32 = thread
    .snapshot(|stepper| stepper.get_state().counter.value)
    .recv_timeout(WAIT)
    .expect("a running loop answers a snapshot");

  assert_eq!(
    value, 3.0,
    "the waiting command was taken by the first tick after resuming"
  );
}

#[test]
fn replays_a_witnessed_session_to_the_same_values_then_takes_live_commands() {
  let recorded: Witnessed = {
    let watched: Watched = Watched::start(None);

    watched.send(TestCommand::Add(2.0));
    wait_past(&watched.thread, 3);
    watched.send(TestCommand::SetSpeed(1.5));
    wait_past(&watched.thread, 8);
    watched.send(TestCommand::Add(-4.0));
    wait_past(&watched.thread, 12);

    // Stopped, the session is whole: nothing more is witnessed.
    drop(watched.thread);

    let witnessed: Witnessed = watched.witnessed.lock().expect("the witness's lock").clone();

    witnessed
  };
  let end: u64 = recorded.log.get_end();

  assert_eq!(recorded.log.len(), 3);
  assert_eq!(recorded.values.len() as u64, end);

  let watched: Watched = Watched::start(Some(recorded.log.clone()));

  assert_eq!(watched.thread.get_replay_end(), Some(end));
  // A live command arriving while the session replays is let go.
  watched.send(TestCommand::Add(100.0));
  wait_past(&watched.thread, end);

  let replayed: Witnessed = watched.read();

  assert_eq!(replayed.values[..end as usize], recorded.values[..]);
  assert_eq!(replayed.log.len(), 3, "{:?}", replayed.log);

  // Past the recorded session's end the loop is played on from.
  watched.send(TestCommand::Add(1_000.0));

  let deadline: Instant = Instant::now() + WAIT;

  while watched.read().log.len() < 4 {
    assert!(Instant::now() < deadline, "the loop never took a live command");
    std::thread::sleep(Duration::from_millis(1));
  }
}
