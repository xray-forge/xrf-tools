use std::sync::Arc;
#[cfg(feature = "mcp")]
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
#[cfg(feature = "mcp")]
use xrf_game_loop::LoopStepper;
use xrf_game_loop::{GameLoop, LoopClock, LoopFrame, LoopStatus};
use xrf_game_movement::{MovementCommand, MovementEvent};
use xrf_game_platform::PlatformEvent;
use xrf_game_settings::GameInputSettings;
use xrf_world::WorldFedCamera;

use crate::host::game_host::GameHost;
use crate::input::game_bindings::GameBindings;
use crate::input::game_controls::GameControls;
use crate::input::game_look::GameLook;
use crate::render::game_view::GameView;
use crate::session::game_session::GameSession;
use crate::sound::game_footsteps::GameFootsteps;
use crate::state::game_command::GameCommand;
use crate::state::game_event::GameEvent;
use crate::state::game_frame::GameFrame;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;

/// A game the main thread runs: its loop stepping the state, the keys that drive it (its tree's controls with the
/// player's bindings over them), what its events play (the player's footsteps), and the host the run keeps for the
/// levels after this one.
pub(crate) struct GameRunning {
  game_loop: GameLoop<GameState>,
  commands: Sender<GameCommand>,
  footsteps: GameFootsteps,
  /// The tree's own controls, which the player's bindings are laid over.
  controls: GameControls,
  bindings: GameBindings,
  #[expect(
    dead_code,
    reason = "kept for the run: the next level the game moves to is opened through it"
  )]
  host: GameHost,
}

impl GameRunning {
  /// Starts the loop over `session`'s state, from now, and shows its level in `view`, the player's view facing where
  /// `look` faces, which starts where the state's player does; its keys bound with `input`'s over the tree's.
  ///
  /// # Errors
  ///
  /// Returns why the loop's thread cannot be started.
  pub(crate) fn start(
    session: GameSession,
    view: &GameView,
    look: &Arc<GameLook>,
    input: &GameInputSettings,
  ) -> XrfResult<Self> {
    let GameSession {
      host,
      source,
      state,
      footsteps,
      controls,
    } = session;
    let clock: LoopClock = LoopClock::new(Instant::now(), LoopClock::DEFAULT_TICK_RATE);

    look.face(state.movement.get_heading());

    let game_loop: GameLoop<GameState> = GameLoop::start(LoopStepper::new(clock.get_step(), state), clock)?;

    view.show(
      source,
      Arc::new(GameViewFeed::new(
        Arc::clone(game_loop.get_feed()),
        Arc::clone(look),
        WorldFedCamera::default(),
      )),
    );

    Ok(Self {
      commands: game_loop.get_sender(),
      game_loop,
      footsteps,
      bindings: GameBindings::new(&controls, input),
      controls,
      host,
    })
  }

  /// The movement command a platform event asks for by the keys bound, none for one no binding reads.
  pub(crate) fn read_input(&self, event: &PlatformEvent) -> Option<MovementCommand> {
    self.bindings.read(event)
  }

  /// Binds the keys again with `input`'s over the tree's controls, as the player's bindings changed.
  pub(crate) fn rebind(&mut self, input: &GameInputSettings) {
    self.bindings = GameBindings::new(&self.controls, input);
  }

  /// The loop's latest frame, none before its first.
  pub(crate) fn get_latest(&self) -> Option<Arc<LoopFrame<GameFrame>>> {
    self.game_loop.get_feed().get_latest()
  }

  /// Sends a command to the loop, answering whether it still takes them.
  pub(crate) fn send(&self, command: GameCommand) -> bool {
    if self.commands.send(command).is_err() {
      log::error!("The game loop stopped taking commands");

      return false;
    }

    true
  }

  /// Reads the state with `read` between two of the loop's ticks, its answer on the receiver.
  #[cfg(feature = "mcp")]
  pub(crate) fn snapshot<R: Send + 'static>(
    &self,
    read: impl FnOnce(&LoopStepper<GameState>) -> R + Send + 'static,
  ) -> Receiver<R> {
    self.game_loop.snapshot(read)
  }

  /// Whether the loop still steps.
  ///
  /// # Errors
  ///
  /// Returns why it stopped: the fault a step met, or the thread's end.
  pub(crate) fn check(&self) -> XrfResult<()> {
    match self.game_loop.get_feed().get_status() {
      LoopStatus::Running => Ok(()),
      LoopStatus::Faulted { tick, message } => Err(XrfError::new_unexpected_error(format!(
        "the game loop faulted at tick {tick}: {message}"
      ))),
      LoopStatus::Stopped => Err(XrfError::new_unexpected_error("the game loop stopped")),
    }
  }

  /// Plays what the loop's steps told since last taken, in order: each footstep's sound in `view`. An object switching
  /// shows nothing yet, as nothing draws the objects.
  pub(crate) fn play_events(&mut self, view: &GameView) {
    for told in self.game_loop.get_events().take() {
      match told.event {
        GameEvent::Movement(MovementEvent::Step(step)) => {
          if let Some(sound) = self.footsteps.hear(&step) {
            view.play_sound(sound);
          }
        }
        GameEvent::Alife(_) => {}
      }
    }
  }
}
