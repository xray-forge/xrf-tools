/// What a key does to a flying camera.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FlyKey {
  Forward,
  Back,
  Left,
  Right,
  Up,
  Down,
  /// Held to move faster, which a level the size of Zaton needs to cross at all.
  Fast,
  TurnLeft,
  TurnRight,
  LookUp,
  LookDown,
}

impl FlyKey {
  /// What a `KeyboardEvent.code` does: WASD and the numpad move the camera, `E` and `Q` or the page keys raise and lower
  /// it, and the arrow keys turn it.
  pub(crate) fn from_code(code: &str) -> Option<Self> {
    Some(match code {
      "ArrowDown" => Self::LookDown,
      "ArrowLeft" => Self::TurnLeft,
      "ArrowRight" => Self::TurnRight,
      "ArrowUp" => Self::LookUp,
      "KeyA" | "Numpad4" => Self::Left,
      "KeyD" | "Numpad6" => Self::Right,
      "KeyE" | "Numpad9" | "PageUp" => Self::Up,
      "KeyQ" | "Numpad7" | "PageDown" => Self::Down,
      "KeyS" | "Numpad2" => Self::Back,
      "KeyW" | "Numpad8" => Self::Forward,
      "ShiftLeft" | "ShiftRight" => Self::Fast,
      _ => return None,
    })
  }
}
