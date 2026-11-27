use xrf_game_movement::MovementAction;
use xrf_game_settings::GameBindingSettings;

/// An action a key is bound to, by X-Ray's name for it (`default_controls.ltx`, `bind <action> <key>`), as the tree,
/// the settings file and the console's `bind` name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameAction {
  Forward,
  Back,
  StrafeLeft,
  StrafeRight,
  Jump,
  Crouch,
  Accel,
  SprintToggle,
  /// Switching between walking and flight, the game's own, which Call of Pripyat binds to nothing.
  Noclip,
}

impl GameAction {
  /// Every action the game binds.
  pub const ALL: [Self; 9] = [
    Self::Forward,
    Self::Back,
    Self::StrafeLeft,
    Self::StrafeRight,
    Self::Jump,
    Self::Crouch,
    Self::Accel,
    Self::SprintToggle,
    Self::Noclip,
  ];

  /// Its X-Ray name.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::Forward => "forward",
      Self::Back => "back",
      Self::StrafeLeft => "lstrafe",
      Self::StrafeRight => "rstrafe",
      Self::Jump => "jump",
      Self::Crouch => "crouch",
      Self::Accel => "accel",
      Self::SprintToggle => "sprint_toggle",
      Self::Noclip => "noclip",
    }
  }

  /// The action X-Ray names `name`, none for one the game does not bind.
  pub fn of_name(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|action| action.get_name() == name.trim())
  }

  /// The movement it drives while held, none for one that acts once as pressed.
  pub fn get_movement(self) -> Option<MovementAction> {
    match self {
      Self::Forward => Some(MovementAction::Forward),
      Self::Back => Some(MovementAction::Back),
      Self::StrafeLeft => Some(MovementAction::Left),
      Self::StrafeRight => Some(MovementAction::Right),
      Self::Jump => Some(MovementAction::Up),
      Self::Crouch => Some(MovementAction::Down),
      Self::Accel => Some(MovementAction::Accel),
      Self::SprintToggle => Some(MovementAction::Sprint),
      Self::Noclip => None,
    }
  }

  /// The key the player bound it to in `bindings`, by its SDL name, none where the tree's binding stands.
  pub fn get_setting(self, bindings: &GameBindingSettings) -> Option<&str> {
    match self {
      Self::Forward => bindings.forward.as_deref(),
      Self::Back => bindings.back.as_deref(),
      Self::StrafeLeft => bindings.lstrafe.as_deref(),
      Self::StrafeRight => bindings.rstrafe.as_deref(),
      Self::Jump => bindings.jump.as_deref(),
      Self::Crouch => bindings.crouch.as_deref(),
      Self::Accel => bindings.accel.as_deref(),
      Self::SprintToggle => bindings.sprint_toggle.as_deref(),
      Self::Noclip => bindings.noclip.as_deref(),
    }
  }
}
