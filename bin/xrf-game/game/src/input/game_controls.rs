use std::collections::BTreeMap;

use xrf_game_config::{ConfigIssue, ConfigIssues};
use xrf_game_platform::PlatformScancode;

use crate::input::game_action::GameAction;
use crate::input::game_key_names::GameKeyNames;

/// The tree's own controls, each action's main key and its second, as its `configs\default_controls.ltx` binds them
/// (`bind <action> <key>`, `bind_sec <action> <key>`, X-Ray's key names), so a mod's controls apply. The keys the
/// player bound in its place are the settings' (`GameInputSettings`), laid over these by `GameBindings`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameControls {
  primary: BTreeMap<GameAction, PlatformScancode>,
  secondary: BTreeMap<GameAction, PlatformScancode>,
}

impl Default for GameControls {
  /// Call of Pripyat's own: W and S as `forward` and `back`, A and D as `lstrafe` and `rstrafe`, Space as `jump`,
  /// Left Ctrl as `crouch`, Left Shift as `accel`, X as `sprint_toggle`; and N for the game's `noclip`.
  fn default() -> Self {
    Self {
      primary: BTreeMap::from([
        (GameAction::Forward, PlatformScancode(26)),
        (GameAction::Back, PlatformScancode(22)),
        (GameAction::StrafeLeft, PlatformScancode(4)),
        (GameAction::StrafeRight, PlatformScancode(7)),
        (GameAction::Jump, PlatformScancode(44)),
        (GameAction::Crouch, PlatformScancode(224)),
        (GameAction::Accel, PlatformScancode(225)),
        (GameAction::SprintToggle, PlatformScancode(27)),
        (GameAction::Noclip, Self::NOCLIP),
      ]),
      secondary: BTreeMap::new(),
    }
  }
}

impl GameControls {
  /// The tree's controls, by its path under `configs`.
  pub const FILE: &'static str = "default_controls.ltx";

  /// N, the game's key for `noclip` where the tree binds none, as no tree of X-Ray's does.
  const NOCLIP: PlatformScancode = PlatformScancode(17);

  /// The controls `text`, the tree's `default_controls.ltx`, binds, as X-Ray's console runs its lines: each `bind`
  /// and `bind_sec` of an action the game binds sets its key; the file's other lines (other actions, `unbindall`,
  /// comments) are not the game's to read. A key it names that the game cannot read is an issue in `issues`, and the
  /// action stays unbound there.
  pub fn read(text: &str, issues: &mut ConfigIssues) -> Self {
    let mut controls: Self = Self {
      primary: BTreeMap::new(),
      secondary: BTreeMap::new(),
    };

    for line in text.lines() {
      let command: &str = line.split(';').next().unwrap_or_default();
      let words: Vec<&str> = command.split_whitespace().collect();
      let [verb, action, key, ..] = words.as_slice() else {
        continue;
      };
      let is_secondary: bool = match *verb {
        "bind" => false,
        "bind_sec" => true,
        _ => continue,
      };
      let Some(action) = GameAction::of_name(action) else {
        continue;
      };
      let Some(scancode) = GameKeyNames::to_sdl(key).and_then(|name| PlatformScancode::from_name(&name)) else {
        issues.push(ConfigIssue {
          file: String::from(Self::FILE),
          section: None,
          key: Some(String::from(*verb)),
          message: format!("binds {} to '{key}', which is no key the game reads", action.get_name()),
        });
        continue;
      };

      if is_secondary {
        controls.secondary.insert(action, scancode);
      } else {
        controls.primary.insert(action, scancode);
      }
    }

    controls.primary.entry(GameAction::Noclip).or_insert(Self::NOCLIP);
    controls
  }

  /// The main key the tree binds `action` to.
  pub fn get_primary(&self, action: GameAction) -> Option<PlatformScancode> {
    self.primary.get(&action).copied()
  }

  /// The second key the tree binds `action` to.
  pub fn get_secondary(&self, action: GameAction) -> Option<PlatformScancode> {
    self.secondary.get(&action).copied()
  }
}
