use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use xrf_error::XrfResult;

use crate::clock::loop_clock::LoopClock;
use crate::feed::loop_feed::LoopFeed;
use crate::loop_stepper::LoopStepper;
use crate::state::loop_state::LoopState;

/// Ticks the thread may fall behind its clock, a quarter of a second at the default rate, before the clock gives up
/// that time rather than stepping through it all at once.
const MOST_BEHIND: u64 = 25;

/// The game loop: the game state stepped on its own thread at its clock's rate. It sleeps to each tick's instant,
/// takes the commands that arrived, steps, and publishes the frame. Dropping it stops the thread and waits for it.
pub struct GameLoop<S: LoopState> {
  sender: Sender<S::Command>,
  feed: Arc<LoopFeed<S::Frame>>,
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
    let feed: Arc<LoopFeed<S::Frame>> = Arc::new(LoopFeed::default());
    let is_stopping: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    feed.publish(clock.get_instant(stepper.get_time().tick), stepper.get_frame());

    let handle: JoinHandle<()> = thread::Builder::new().name(String::from("xrf-game-loop")).spawn({
      let feed: Arc<LoopFeed<S::Frame>> = Arc::clone(&feed);
      let is_stopping: Arc<AtomicBool> = Arc::clone(&is_stopping);

      move || Self::run(stepper, clock, &receiver, &feed, &is_stopping)
    })?;

    log::info!("Game loop started at {:?} a tick", clock.get_step());

    Ok(Self {
      sender,
      feed,
      is_stopping,
      handle: Some(handle),
    })
  }

  /// A sender of commands, which the next tick takes; one sent after the thread stopped is dropped.
  pub fn get_sender(&self) -> Sender<S::Command> {
    self.sender.clone()
  }

  /// The frames it publishes.
  pub fn get_feed(&self) -> &Arc<LoopFeed<S::Frame>> {
    &self.feed
  }

  fn run(
    mut stepper: LoopStepper<S>,
    mut clock: LoopClock,
    receiver: &Receiver<S::Command>,
    feed: &LoopFeed<S::Frame>,
    is_stopping: &AtomicBool,
  ) {
    let mut commands: Vec<S::Command> = Vec::new();

    while !is_stopping.load(Ordering::Acquire) {
      let due: Instant = clock.get_instant(stepper.get_time().tick + 1);
      let now: Instant = Instant::now();

      // A sleep rather than a timed receive, whose wait on Windows is coarser; commands are only taken at a tick's
      // start anyway.
      if due > now {
        thread::sleep(due - now);
      }

      commands.extend(receiver.try_iter());
      stepper.step(commands.drain(..));

      let tick: u64 = stepper.get_time().tick;

      feed.publish(clock.get_instant(tick), stepper.get_frame());

      let now: Instant = Instant::now();
      let behind: u64 = clock.get_tick_at(now).saturating_sub(tick);

      if behind > MOST_BEHIND {
        log::warn!("Game loop fell {behind} ticks behind its clock, which gives that time up");
        clock.slip(tick, now);
      }
    }

    log::info!("Game loop stopped at tick {}", stepper.get_time().tick);
  }
}

impl<S: LoopState> Drop for GameLoop<S> {
  fn drop(&mut self) {
    self.is_stopping.store(true, Ordering::Release);

    if let Some(handle) = self.handle.take()
      && handle.join().is_err()
    {
      log::error!("The stepper thread panicked");
    }
  }
}
