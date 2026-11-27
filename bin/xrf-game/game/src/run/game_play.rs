#[cfg(feature = "mcp")]
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;

#[cfg(feature = "mcp")]
use xrf_game_loop::LoopFeed;
use xrf_game_platform::{Platform, PlatformEvent, PlatformEventKind, PlatformScancode, PlatformWindow};

#[cfg(feature = "mcp")]
use crate::control::game_control::GameControl;
use crate::input::game_bindings::GameBindings;
use crate::render::game_view::GameView;
use crate::state::game_command::GameCommand;
#[cfg(feature = "mcp")]
use crate::state::game_frame::GameFrame;

/// The game while it is played: the platform's events read through the bindings into the loop's commands, the view fit
/// to the window, and the control endpoint's requests answered between frames, until the player or an agent quits.
pub(crate) struct GamePlay<'a> {
  pub window: &'a PlatformWindow,
  pub view: &'a GameView,
  pub commands: Sender<GameCommand>,
  pub bindings: GameBindings,
  /// What the loop publishes, which the control endpoint's status reads.
  #[cfg(feature = "mcp")]
  pub feed: Arc<LoopFeed<GameFrame>>,
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
  pub(crate) fn run(&self, platform: &mut Platform) {
    let mut events: Vec<PlatformEvent> = Vec::new();

    loop {
      events.clear();
      platform.poll(&mut events);

      if !events.iter().all(|event| self.take_event(event)) {
        return;
      }

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
  fn take_event(&self, event: &PlatformEvent) -> bool {
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
}
