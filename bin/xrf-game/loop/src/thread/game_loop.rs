use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use xrf_error::XrfResult;

use crate::clock::loop_clock::LoopClock;
use crate::feed::loop_events::LoopEvents;
use crate::feed::loop_feed::LoopFeed;
use crate::feed::loop_status::LoopStatus;
use crate::loop_stepper::LoopStepper;
use crate::state::loop_state::LoopState;
use crate::thread::loop_control::LoopControl;

/// Ticks the thread may fall behind its clock, a quarter of a second at the default rate, before the clock gives up
/// that time rather than stepping through it all at once.
const MOST_BEHIND: u64 = 25;

/// How long a paused thread waits for the next control before looking whether it should stop.
const PAUSED_WAIT: Duration = Duration::from_millis(10);

/// The game loop: the game state stepped on its own thread at its clock's rate. It sleeps to each tick's instant,
/// takes what the game asked of the loop and the commands that arrived, steps, publishes the frame and queues the
/// step's events. It may be paused, and read between two ticks paused or not. A step's fault stops it, once logged, its
/// status saying why. Dropping it stops the thread and waits for it.
pub struct GameLoop<S: LoopState> {
  sender: Sender<S::Command>,
  control: Sender<LoopControl<S>>,
  feed: Arc<LoopFeed<S::Frame>>,
  events: Arc<LoopEvents<S::Event>>,
  is_stopping: Arc<AtomicBool>,
  handle: Option<JoinHandle<()>>,
}

impl<S: LoopState> GameLoop<S> {
  /// Starts stepping `stepper` from the tick after its own, by `clock`, its present frame published at once.
  ///
  /// # Errors
  ///
  /// Returns why the thread cannot be started.
  pub fn start(stepper: LoopStepper<S>, clock: LoopClock) -> XrfResult<Self> {
    let (sender, receiver) = mpsc::channel::<S::Command>();
    let (control, controls) = mpsc::channel::<LoopControl<S>>();
    let feed: Arc<LoopFeed<S::Frame>> = Arc::new(LoopFeed::default());
    let events: Arc<LoopEvents<S::Event>> = Arc::new(LoopEvents::default());
    let is_stopping: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    feed.publish(clock.get_instant(stepper.get_time().tick), stepper.get_frame());

    let handle: JoinHandle<()> = thread::Builder::new().name(String::from("xrf-game-loop")).spawn({
      let outlets: LoopOutlets<S> = LoopOutlets {
        feed: Arc::clone(&feed),
        events: Arc::clone(&events),
      };
      let is_stopping: Arc<AtomicBool> = Arc::clone(&is_stopping);

      move || {
        LoopThread {
          stepper,
          clock,
          is_paused: false,
        }
        .run(&receiver, &controls, &outlets, &is_stopping);
      }
    })?;

    log::info!("Game loop started at {:?} a tick", clock.get_step());

    Ok(Self {
      sender,
      control,
      feed,
      events,
      is_stopping,
      handle: Some(handle),
    })
  }

  /// A sender of commands, which the next tick takes; one sent after the thread stopped is dropped.
  pub fn get_sender(&self) -> Sender<S::Command> {
    self.sender.clone()
  }

  /// The frames it publishes and whether it still steps.
  pub fn get_feed(&self) -> &Arc<LoopFeed<S::Frame>> {
    &self.feed
  }

  /// The events its steps told, waiting to be taken.
  pub fn get_events(&self) -> &Arc<LoopEvents<S::Event>> {
    &self.events
  }

  /// Stops stepping before the next tick, until resumed; commands sent meanwhile wait for it.
  pub fn pause(&self) {
    self.send_control(LoopControl::Pause);
  }

  /// Steps again from now, the time paused given up rather than stepped through.
  pub fn resume(&self) {
    self.send_control(LoopControl::Resume);
  }

  /// Reads the state with `read` between two ticks, paused or not, its answer on the receiver; none comes once the
  /// thread stopped.
  pub fn snapshot<R: Send + 'static>(&self, read: impl FnOnce(&LoopStepper<S>) -> R + Send + 'static) -> Receiver<R> {
    let (reply, answer) = mpsc::channel::<R>();

    self.send_control(LoopControl::Snapshot(Box::new(move |stepper: &LoopStepper<S>| {
      let _ = reply.send(read(stepper));
    })));

    answer
  }

  fn send_control(&self, control: LoopControl<S>) {
    // A thread that stopped takes nothing more, and a snapshot's receiver then answers none.
    let _ = self.control.send(control);
  }
}

impl<S: LoopState> Drop for GameLoop<S> {
  fn drop(&mut self) {
    self.is_stopping.store(true, Ordering::Release);

    if let Some(handle) = self.handle.take()
      && handle.join().is_err()
    {
      log::error!("The game loop's thread panicked");
    }
  }
}

/// Where the loop's thread puts what it publishes: the frames and status, and the events.
struct LoopOutlets<S: LoopState> {
  feed: Arc<LoopFeed<S::Frame>>,
  events: Arc<LoopEvents<S::Event>>,
}

/// Notes how the thread ended once it does, however it does: a panic is a fault, and anything else a stop, unless a
/// fault was noted first.
struct LoopEnding<'a, F> {
  feed: &'a LoopFeed<F>,
  tick: u64,
}

impl<F> Drop for LoopEnding<'_, F> {
  fn drop(&mut self) {
    self.feed.end(if thread::panicking() {
      LoopStatus::Faulted {
        tick: self.tick,
        message: String::from("the game loop's thread panicked"),
      }
    } else {
      LoopStatus::Stopped
    });
  }
}

/// What the loop's thread keeps between ticks.
struct LoopThread<S: LoopState> {
  stepper: LoopStepper<S>,
  clock: LoopClock,
  is_paused: bool,
}

impl<S: LoopState> LoopThread<S> {
  fn run(
    mut self,
    receiver: &Receiver<S::Command>,
    controls: &Receiver<LoopControl<S>>,
    outlets: &LoopOutlets<S>,
    is_stopping: &AtomicBool,
  ) {
    let mut ending: LoopEnding<'_, S::Frame> = LoopEnding {
      feed: &outlets.feed,
      tick: self.stepper.get_time().tick,
    };
    let mut commands: Vec<S::Command> = Vec::new();

    while !is_stopping.load(Ordering::Acquire) {
      if self.is_paused {
        match controls.recv_timeout(PAUSED_WAIT) {
          Ok(control) => self.take(control),
          Err(RecvTimeoutError::Timeout) => {}
          Err(RecvTimeoutError::Disconnected) => break,
        }

        continue;
      }

      let due: Instant = self.clock.get_instant(self.stepper.get_time().tick + 1);
      let now: Instant = Instant::now();

      // A sleep rather than a timed receive, whose wait on Windows is coarser; what arrives is only taken at a tick's
      // start anyway.
      if due > now {
        thread::sleep(due - now);
      }

      for control in controls.try_iter() {
        self.take(control);
      }

      if self.is_paused {
        continue;
      }

      let from: u64 = self.stepper.get_time().tick;

      commands.extend(receiver.try_iter());

      match self.stepper.step(commands.drain(..)) {
        Ok(events) => outlets.events.push(from, events),
        Err(error) => {
          log::error!("Game loop faulted at tick {from} and steps no further: {error}");
          outlets.feed.end(LoopStatus::Faulted {
            tick: from,
            message: error.to_string(),
          });

          break;
        }
      }

      let tick: u64 = self.stepper.get_time().tick;

      ending.tick = tick;
      outlets
        .feed
        .publish(self.clock.get_instant(tick), self.stepper.get_frame());

      let now: Instant = Instant::now();
      let behind: u64 = self.clock.get_tick_at(now).saturating_sub(tick);

      if behind > MOST_BEHIND {
        log::warn!("Game loop fell {behind} ticks behind its clock, which gives that time up");
        self.clock.slip(tick, now);
      }
    }

    log::info!("Game loop stopped at tick {}", self.stepper.get_time().tick);
  }

  fn take(&mut self, control: LoopControl<S>) {
    match control {
      LoopControl::Pause => {
        if !self.is_paused {
          log::info!("Game loop paused at tick {}", self.stepper.get_time().tick);
        }

        self.is_paused = true;
      }
      LoopControl::Resume => {
        if self.is_paused {
          let tick: u64 = self.stepper.get_time().tick;

          log::info!("Game loop resumed at tick {tick}");
          // The next tick falls due a step from now, as though the pause never was.
          self.clock.slip(tick, Instant::now());
        }

        self.is_paused = false;
      }
      LoopControl::Snapshot(read) => read(&self.stepper),
    }
  }
}
