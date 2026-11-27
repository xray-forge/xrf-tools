use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "mcp")]
use std::time::Instant;

use xrf_error::XrfResult;
use xrf_game_console::{ConsoleAnswer, ConsoleChoice};
use xrf_game_developer::{DeveloperLoading, DeveloperSuggestion, DeveloperView};
use xrf_game_loop::LoopFrame;
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
use crate::run::game_flow::GameFlow;
use crate::run::game_flow_stage::GameFlowStage;
use crate::run::game_running::GameRunning;
use crate::run::game_startup::GameStartup;
use crate::state::game_command::GameCommand;
use crate::state::game_frame::GameFrame;

/// The main thread from the window's opening to the game's end: the platform's events read through the developer layer
/// and the bindings into the loop's commands, the mouse turning the look the next drawn frame faces (sent to the loop
/// once a frame it changed), the view fit to the window, the console's lines run and the log's records shown in it, the
/// flow moved on (a load's view drawn while it runs, then the game played), what the loop's steps told played (the
/// player's footsteps), the developer layer drawn over it, and the control endpoint's requests answered between frames,
/// until the player, a console line or an agent quits, or the loop stops stepping.
pub(crate) struct GamePlay<'a> {
  pub window: &'a mut PlatformWindow,
  pub view: &'a GameView,
  pub bindings: GameBindings,
  pub developer: GameDeveloper,
  pub console: GameConsole,
  /// The log's records the console shows, taken as they come.
  pub log: Arc<LogTail>,
  /// The control endpoint, where the game was started with `--mcp` or `--listen`.
  #[cfg(feature = "mcp")]
  pub control: Option<GameControl>,
  /// Whether a console line asked the game to quit.
  pub is_quitting: bool,
  /// How the hidden window first shows.
  pub startup: GameStartup,
  /// The load running and the game played.
  pub flow: GameFlow,
}

impl GamePlay<'_> {
  /// The key that quits until the game has a menu: Escape's place.
  const QUIT_KEY: PlatformScancode = PlatformScancode(41);

  /// How long the main thread waits between reads of the platform's events.
  const POLL_WAIT: Duration = Duration::from_millis(1);

  /// Plays until the player or an agent quits.
  ///
  /// # Errors
  ///
  /// Returns why the game being loaded did not open, or its loop did not start.
  pub(crate) fn run(&mut self, platform: &mut Platform) -> XrfResult<()> {
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
        return Ok(());
      }

      // The view already faces the look the render thread reads; the simulation is sent it whole, once a frame.
      if let Some(heading) = self.flow.get_look().take_changed() {
        self.send(GameCommand::Movement(MovementCommand::Face(heading)));
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
        return Ok(());
      }

      let view: DeveloperView = self.get_developer_view();
      let loading: Option<DeveloperLoading> = self.flow.advance(self.view, &mut self.developer)?;
      let latest: Option<Arc<LoopFrame<GameFrame>>> = self.flow.get_running().and_then(GameRunning::get_latest);

      if let Some(running) = self.flow.get_running_mut() {
        running.check()?;
        running.play_events(self.view);
      }

      self
        .developer
        .send(self.view, view, latest.as_deref(), loading.as_ref());

      // The endpoint is taken out while it serves, since a console request runs a line through play.
      #[cfg(feature = "mcp")]
      if let Some(control) = self.control.take() {
        let is_playing: bool = control.serve(self);

        self.control = Some(control);

        if !is_playing || self.is_quitting {
          return Ok(());
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
        GameConsoleEffect::Movement(command) => {
          self.send(GameCommand::Movement(command));
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

  /// Sends a command to the game's loop, answering whether one took it: none while the game opens.
  pub(crate) fn send(&self, command: GameCommand) -> bool {
    let Some(running) = self.flow.get_running() else {
      log::warn!("No game runs yet to take {command:?}");

      return false;
    };

    running.send(command)
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
      _ if self.flow.get_stage() == GameFlowStage::Loading => true,
      // The mouse turns the look at once, which the next drawn frame faces.
      PlatformEventKind::MouseMoved { dx, dy, .. } => {
        self.flow.get_look().turn(*dx, *dy);

        true
      }
      _ => self
        .bindings
        .read(event)
        .is_none_or(|command| self.send(GameCommand::Movement(command))),
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
