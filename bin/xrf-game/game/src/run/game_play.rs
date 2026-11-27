use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;
#[cfg(feature = "mcp")]
use std::time::Instant;

use xrf_game_developer::DeveloperView;
use xrf_game_loop::{LoopFeed, LoopFrame};
use xrf_game_movement::{MovementCommand, MovementInput, MovementInputKind};
use xrf_game_platform::{
  Platform, PlatformEvent, PlatformEventKind, PlatformScancode, PlatformSize, PlatformWindow, PlatformWindowCommand,
};

#[cfg(feature = "mcp")]
use crate::control::game_control::GameControl;
use crate::developer::game_developer::GameDeveloper;
use crate::input::game_bindings::GameBindings;
use crate::render::game_view::GameView;
use crate::sound::game_footsteps::GameFootsteps;
use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;

/// The game while it is played: the platform's events read through the developer layer and the bindings into the
/// loop's commands, the view fit to the window, the player's footsteps played from the loop's latest frame, the
/// developer layer drawn over it, and the control endpoint's requests answered between frames, until the player or an
/// agent quits.
pub(crate) struct GamePlay<'a> {
  pub window: &'a mut PlatformWindow,
  pub view: &'a GameView,
  pub commands: Sender<GameCommand>,
  pub bindings: GameBindings,
  /// What the loop publishes, which the developer layer's readout and the control endpoint's status read.
  pub feed: Arc<LoopFeed<GameFrame>>,
  pub developer: GameDeveloper,
  pub footsteps: GameFootsteps,
  /// The control endpoint, where the game was started with `--mcp`.
  #[cfg(feature = "mcp")]
  pub control: Option<GameControl>,
}

impl GamePlay<'_> {
  /// The key that quits until the game has a menu: Escape's place.
  const QUIT_KEY: PlatformScancode = PlatformScancode(41);

  /// How long the main thread waits between reads of the platform's events.
  const POLL_WAIT: Duration = Duration::from_millis(1);

  /// Plays until the player or an agent quits.
  pub(crate) fn run(&mut self, platform: &mut Platform) {
    let mut events: Vec<PlatformEvent> = Vec::new();

    loop {
      events.clear();
      platform.poll(&mut events);

      #[cfg(feature = "mcp")]
      if let Some(control) = &self.control {
        control.take_inputs(Instant::now(), platform.get_time(), &mut events);
      }

      if !events.iter().all(|event| self.take_event(event)) {
        return;
      }

      let view: DeveloperView = self.get_developer_view();
      let latest: Option<LoopFrame<GameFrame>> = self.feed.get_latest();

      if let Some(step) = self
        .footsteps
        .hear(latest.as_ref().and_then(|frame| frame.state.footing.as_ref()))
      {
        self.view.play_sound(step);
      }

      self.developer.send(self.view, view, latest);

      #[cfg(feature = "mcp")]
      if let Some(control) = &self.control
        && !control.serve(self)
      {
        return;
      }

      std::thread::sleep(Self::POLL_WAIT);
    }
  }

  /// Sends a command to the loop, answering whether it still takes them.
  pub(crate) fn send(&self, command: GameCommand) -> bool {
    if self.commands.send(command).is_err() {
      log::error!("The game loop stopped taking commands");

      return false;
    }

    true
  }

  /// Takes one platform event, answering whether the game plays on.
  fn take_event(&mut self, event: &PlatformEvent) -> bool {
    let was_open: bool = self.developer.is_open();

    if self.developer.take(event, self.get_developer_view()) {
      return self.developer.is_open() == was_open || self.toggle_developer(event);
    }

    match &event.kind {
      PlatformEventKind::Quit
      | PlatformEventKind::Key {
        scancode: Self::QUIT_KEY,
        is_down: true,
        ..
      } => false,
      PlatformEventKind::Resized { .. } => {
        self.view.fit(self.window);

        true
      }
      _ => self.bindings.read(event).is_none_or(|command| self.send(command)),
    }
  }

  /// Hands the keyboard and mouse to the console as it opens, back to the game as it closes; held actions are let go,
  /// since their releases go to the console. Answers whether the game plays on.
  fn toggle_developer(&mut self, event: &PlatformEvent) -> bool {
    let is_open: bool = self.developer.is_open();

    for command in [
      PlatformWindowCommand::GrabCursor { is_grabbed: !is_open },
      PlatformWindowCommand::AcceptText { is_accepting: is_open },
    ] {
      if let Err(error) = self.window.apply(command) {
        log::warn!("The window did not take {command:?}: {error}");
      }
    }

    self.send(GameCommand::Movement(MovementCommand::Input(MovementInput {
      at: event.at,
      kind: MovementInputKind::ReleaseAll,
    })))
  }

  /// The window as the developer layer draws over it this frame.
  fn get_developer_view(&self) -> DeveloperView {
    let PlatformSize { width, height } = self.window.get_size();

    DeveloperView {
      width,
      height,
      scale: self.window.get_scale(),
      pixel_density: self.window.get_pixel_density(),
    }
  }
}
