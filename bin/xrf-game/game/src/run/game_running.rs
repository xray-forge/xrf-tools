use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_game_loop::{GameLoop, LoopClock, LoopFrame, LoopStepper};
use xrf_world::WorldFedCamera;

use crate::host::game_host::GameHost;
use crate::render::game_view::GameView;
use crate::session::game_session::GameSession;
use crate::sound::game_footsteps::GameFootsteps;
use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;

/// A game the main thread runs: its loop stepping the state, the player's footsteps played from its frames, and the
/// host the run keeps for the levels after this one.
pub(crate) struct GameRunning {
  game_loop: GameLoop<GameState>,
  commands: Sender<GameCommand>,
  footsteps: GameFootsteps,
  #[expect(
    dead_code,
    reason = "kept for the run: the next level the game moves to is opened through it"
  )]
  host: GameHost,
}

impl GameRunning {
  /// Starts the loop over `session`'s state, from now, and shows its level in `view`.
  ///
  /// # Errors
  ///
  /// Returns why the loop's thread cannot be started.
  pub(crate) fn start(session: GameSession, view: &GameView) -> XrfResult<Self> {
    let GameSession {
      host,
      source,
      state,
      footsteps,
    } = session;
    let clock: LoopClock = LoopClock::new(Instant::now(), LoopClock::DEFAULT_TICK_RATE);
    let game_loop: GameLoop<GameState> = GameLoop::start(LoopStepper::new(clock.get_step(), state), clock)?;

    view.show(
      source,
      Arc::new(GameViewFeed::new(
        Arc::clone(game_loop.get_feed()),
        WorldFedCamera::default(),
      )),
    );

    Ok(Self {
      commands: game_loop.get_sender(),
      game_loop,
      footsteps,
      host,
    })
  }

  /// The loop's latest frame, none before its first.
  pub(crate) fn get_latest(&self) -> Option<LoopFrame<GameFrame>> {
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

  /// Plays the step the player's latest `frame` finishes, if it finishes one.
  pub(crate) fn play_footsteps(&mut self, view: &GameView, frame: Option<&LoopFrame<GameFrame>>) {
    if let Some(step) = self
      .footsteps
      .hear(frame.and_then(|frame| frame.state.footing.as_ref()))
    {
      view.play_sound(step);
    }
  }
}
