use xrf_game_console::{ConsoleAnswer, ConsoleCompletion};
use xrf_game_platform::PlatformWindow;
use xrf_game_settings::GameSettingsStore;

use crate::console::game_console::GameConsole;
use crate::console::game_console_effect::GameConsoleEffect;
use crate::render::game_view::GameView;
use crate::run::game_flow::GameFlow;
use crate::state::game_command::GameCommand;

/// What every presentation layer acts on beside its own state: the window and the view drawn in it, the run's settings,
/// the flow (the load running, the game played, the player's look), and the console every layer runs its lines
/// through. Play owns it and lends it to each layer in turn, so no layer holds another.
pub(crate) struct GameLayerContext<'a> {
  pub window: &'a mut PlatformWindow,
  pub view: &'a GameView,
  /// The settings the run plays with: lent to the console for each line, what changed applied by play once a frame.
  pub settings: GameSettingsStore,
  /// The load running and the game played.
  pub flow: GameFlow,
  console: GameConsole,
  /// The lines run through the console and their answers, not yet taken by the layer that shows them.
  console_lines: Vec<String>,
  /// Whether a layer asked the game to quit.
  is_quitting: bool,
}

impl<'a> GameLayerContext<'a> {
  /// The context of a run in `window`, drawn in `view`, playing with `settings`, its game the one `flow` loads.
  pub(crate) fn new(
    window: &'a mut PlatformWindow,
    view: &'a GameView,
    settings: GameSettingsStore,
    flow: GameFlow,
  ) -> Self {
    Self {
      window,
      view,
      settings,
      flow,
      console: GameConsole::default(),
      console_lines: Vec::new(),
      is_quitting: false,
    }
  }

  /// Runs the console line `text`, whichever layer enters it, the drawer or an agent: kept with its answer for the
  /// drawer to show, then what it asks of the game applied.
  pub(crate) fn run_console(&mut self, text: &str) -> ConsoleAnswer {
    let answer: ConsoleAnswer = self.console.execute(&mut self.settings, text);

    self.console_lines.push(format!("> {text}"));
    self.console_lines.extend(answer.lines.iter().cloned());

    for effect in self.console.take_effects() {
      match effect {
        GameConsoleEffect::Movement(command) => {
          self.send(GameCommand::Movement(command));
        }
        GameConsoleEffect::Quit => self.is_quitting = true,
      }
    }

    answer
  }

  /// What the console line `text` may become, as Tab completes it: the line completed, and its choices with their
  /// hints.
  pub(crate) fn complete_console(&mut self, text: &str) -> ConsoleCompletion {
    self.console.complete(&mut self.settings, text)
  }

  /// The lines run through the console since the last take, each with its answer, in order.
  pub(crate) fn take_console_lines(&mut self) -> Vec<String> {
    std::mem::take(&mut self.console_lines)
  }

  /// Sends a command to the game's loop, answering whether one took it: none while the game opens.
  pub(crate) fn send(&self, command: GameCommand) -> bool {
    let Some(running) = self.flow.get_running() else {
      log::warn!("No game runs yet to take {command:?}");

      return false;
    };

    running.send(command)
  }

  /// Ends play once the frame is done.
  pub(crate) fn quit(&mut self) {
    self.is_quitting = true;
  }

  pub(crate) fn is_quitting(&self) -> bool {
    self.is_quitting
  }
}
