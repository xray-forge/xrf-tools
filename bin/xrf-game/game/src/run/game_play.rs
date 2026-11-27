use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use xrf_game_console::{ConsoleAnswer, ConsoleChoice};
use xrf_game_developer::{DeveloperLoading, DeveloperSuggestion, DeveloperView};
use xrf_game_loop::{LoopFeed, LoopFrame};
use xrf_game_movement::{MovementCommand, MovementInput, MovementInputKind};
use xrf_game_platform::{
  Platform, PlatformEvent, PlatformEventKind, PlatformScancode, PlatformSize, PlatformWindow, PlatformWindowCommand,
};
use xrf_logging::LogTail;

use crate::console::game_console::GameConsole;
use crate::console::game_console_effect::GameConsoleEffect;
#[cfg(feature = "mcp")]
use crate::control::game_control::GameControl;
use crate::developer::game_developer::GameDeveloper;
use crate::input::game_bindings::GameBindings;
use crate::render::game_view::GameView;
use crate::run::game_startup::GameStartup;
use crate::sound::game_footsteps::GameFootsteps;
use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;

/// The game while it is played: the platform's events read through the developer layer and the bindings into the
/// loop's commands, the view fit to the window, the console's lines run and the log's records shown in it, the
/// player's footsteps played from the loop's latest frame, the developer layer drawn over it, and the control
/// endpoint's requests answered between frames, until the player, a console line or an agent quits.
pub(crate) struct GamePlay<'a> {
  pub window: &'a mut PlatformWindow,
  pub view: &'a GameView,
  pub commands: Sender<GameCommand>,
  pub bindings: GameBindings,
  /// What the loop publishes, which the developer layer's readout and the control endpoint's status read.
  pub feed: Arc<LoopFeed<GameFrame>>,
  pub developer: GameDeveloper,
  pub console: GameConsole,
  /// The log's records the console shows, taken as they come.
  pub log: Arc<LogTail>,
  pub footsteps: GameFootsteps,
  /// The control endpoint, where the game was started with `--mcp`.
  #[cfg(feature = "mcp")]
  pub control: Option<GameControl>,
  /// Whether a console line asked the game to quit.
  pub is_quitting: bool,
  /// How the game comes up: the hidden window shown, the loading view over the level until it is ready.
  pub startup: GameStartup,
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
      self.startup.show_when_covered(
        self.window,
        self.view,
        self.developer.has_sent(),
        self.developer.is_open(),
      );
      events.clear();
      platform.poll(&mut events);

      #[cfg(feature = "mcp")]
      if let Some(control) = &self.control {
        control.take_inputs(Instant::now(), platform.get_time(), &mut events);
      }

      if !events.iter().all(|event| self.take_event(event)) {
        return;
      }

      for line in self.developer.take_entered() {
        self.run_console(&line);
      }

      if let Some(line) = self.developer.take_completing() {
        self.developer.complete(self.console.complete(&line).line);
      }

      // After a completion, so the line it left is suggested from at once.
      if let Some(line) = self.developer.take_typed() {
        self.suggest_console(&line);
      }

      self.developer.push_lines(self.log.take());

      if self.is_quitting {
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

      let loading: Option<DeveloperLoading> = self.startup.describe_loading(self.view);

      self.developer.send(self.view, view, latest, loading.as_ref());

      // The endpoint is taken out while it serves, since a console request runs a line through play.
      #[cfg(feature = "mcp")]
      if let Some(control) = self.control.take() {
        let is_playing: bool = control.serve(self);

        self.control = Some(control);

        if !is_playing || self.is_quitting {
          return;
        }
      }

      std::thread::sleep(Self::POLL_WAIT);
    }
  }

  /// Runs the console line `text`, as the drawer or the bridge enters it: shown in the drawer with its answer, then
  /// what it asks of the game applied.
  pub(crate) fn run_console(&mut self, text: &str) -> ConsoleAnswer {
    let answer: ConsoleAnswer = self.console.execute(text);

    self
      .developer
      .push_lines(std::iter::once(format!("> {text}")).chain(answer.lines.iter().cloned()));

    for effect in self.console.take_effects() {
      match effect {
        GameConsoleEffect::ApplySettings => self.view.apply_settings(self.console.get_settings()),
        GameConsoleEffect::Command(command) => {
          self.send(command);
        }
        GameConsoleEffect::Quit => self.is_quitting = true,
      }
    }

    answer
  }

  /// Suggests what the drawer's line `text` may become: the console's choices with their hints, none for a line past
  /// its name.
  fn suggest_console(&mut self, text: &str) {
    let suggestions: Vec<DeveloperSuggestion> = self
      .console
      .complete(text)
      .choices
      .into_iter()
      .map(|ConsoleChoice { name, hint }| DeveloperSuggestion { text: name, hint })
      .collect();

    self.developer.suggest(suggestions);
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
      // The player is held while the loading view covers the level.
      _ if self.startup.is_loading() => true,
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
