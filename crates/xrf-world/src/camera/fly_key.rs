/// What a key does to a flying camera.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FlyKey {
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
  pub fn from_code(code: &str) -> Option<Self> {
    Some(match code {
      "ArrowDown" => FlyKey::LookDown,
      "ArrowLeft" => FlyKey::TurnLeft,
      "ArrowRight" => FlyKey::TurnRight,
      "ArrowUp" => FlyKey::LookUp,
      "KeyA" | "Numpad4" => FlyKey::Left,
      "KeyD" | "Numpad6" => FlyKey::Right,
      "KeyE" | "Numpad9" | "PageUp" => FlyKey::Up,
      "KeyQ" | "Numpad7" | "PageDown" => FlyKey::Down,
      "KeyS" | "Numpad2" => FlyKey::Back,
      "KeyW" | "Numpad8" => FlyKey::Forward,
      "ShiftLeft" | "ShiftRight" => FlyKey::Fast,
      _ => return None,
    })
  }
}
