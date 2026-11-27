use std::collections::HashMap;

use xrf_game_movement::{MovementCommand, MovementInput, MovementInputKind};
use xrf_game_platform::{PlatformEvent, PlatformEventKind, PlatformScancode};
use xrf_game_settings::{GameBindingSettings, GameInputSettings};

use crate::input::game_action::GameAction;
use crate::input::game_controls::GameControls;
use crate::input::game_key_names::GameKeyNames;

/// Which key does what: the platform's events read into the game's commands, each action's keys the player's where the
/// settings bind it (`input.bindings`, `input.bindings_secondary`), else the tree's own (`GameControls`). Keys are bound
/// by their place on the keyboard, so the layout never moves them. The mouse turns presentation's look (`GameLook`),
/// not a command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameBindings {
  keys: HashMap<PlatformScancode, GameAction>,
}

impl Default for GameBindings {
  /// Call of Pripyat's controls, nothing rebound.
  fn default() -> Self {
    Self::new(&GameControls::default(), &GameInputSettings::default())
  }
}

impl GameBindings {
  /// The tree's `controls` with the keys `input` binds in their place. A key named that the platform does not know is
  /// logged, and the tree's binding stands.
  pub fn new(controls: &GameControls, input: &GameInputSettings) -> Self {
    let mut keys: HashMap<PlatformScancode, GameAction> = HashMap::new();

    for action in GameAction::ALL {
      let primary: Option<PlatformScancode> =
        Self::find_bound(action, &input.bindings).or_else(|| controls.get_primary(action));
      let secondary: Option<PlatformScancode> =
        Self::find_bound(action, &input.bindings_secondary).or_else(|| controls.get_secondary(action));

      for key in primary.into_iter().chain(secondary) {
        keys.insert(key, action);
      }
    }

    Self { keys }
  }

  /// The key a name stands for, SDL's (`Left Shift`) or X-Ray's (`kLSHIFT`), none for one the platform does not know.
  pub fn find_key(name: &str) -> Option<PlatformScancode> {
    PlatformScancode::from_name(name)
      .or_else(|| GameKeyNames::to_sdl(name).and_then(|sdl| PlatformScancode::from_name(&sdl)))
  }

  /// The movement command an event asks for, none for an event no binding reads. A key's repeats are not presses;
  /// losing focus lets every action go, since its releases will never arrive.
  pub fn read(&self, event: &PlatformEvent) -> Option<MovementCommand> {
    let kind: MovementInputKind = match &event.kind {
      PlatformEventKind::Key {
        scancode,
        is_down,
        is_repeat: false,
      } => {
        let action: GameAction = *self.keys.get(scancode)?;

        match action.get_movement() {
          Some(movement) => MovementInputKind::Action {
            action: movement,
            is_pressed: *is_down,
          },
          // Switching the mode acts once, as pressed.
          None => return is_down.then_some(MovementCommand::ToggleMode),
        }
      }
      PlatformEventKind::Focused { is_focused: false } => MovementInputKind::ReleaseAll,
      _ => return None,
    };

    Some(MovementCommand::Input(MovementInput { at: event.at, kind }))
  }

  /// The key the player bound `action` to in `bindings`, none where they bound none or, logged, one the platform does
  /// not know.
  fn find_bound(action: GameAction, bindings: &GameBindingSettings) -> Option<PlatformScancode> {
    let name: &str = action.get_setting(bindings)?;
    let key: Option<PlatformScancode> = Self::find_key(name);

    if key.is_none() {
      log::warn!(
        "Binding {} to '{name}' left out: no key is named so; the tree's binding stands",
        action.get_name()
      );
    }

    key
  }
}
