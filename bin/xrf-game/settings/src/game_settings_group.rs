/// A group of the game's settings, by the first part of a setting's path: what a change touches, so each part of the
/// game applies only the changes it reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GameSettingsGroup {
  /// What every frame draws with (`render.`).
  Render,
  /// What the game's view draws (`view.`).
  View,
  /// What the view's scene is drawn at and how it is upscaled (`upscaling.`).
  Upscaling,
  /// How the window opens (`window.`), taken only as it opens.
  Window,
  /// How the player's input reads (`input.`): the mouse's sensitivity and the bindings.
  Input,
}

impl GameSettingsGroup {
  /// The group the setting at `path` belongs to, none for a path that names none.
  pub fn of_path(path: &str) -> Option<Self> {
    match path.split('.').next()? {
      "render" => Some(Self::Render),
      "view" => Some(Self::View),
      "upscaling" => Some(Self::Upscaling),
      "window" => Some(Self::Window),
      "input" => Some(Self::Input),
      _ => None,
    }
  }
}
