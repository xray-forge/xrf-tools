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
  /// What a `KeyboardEvent.code` does: WASD, `E` and `Q` move the camera, and the arrow keys turn it.
  pub fn from_code(code: &str) -> Option<Self> {
    Some(match code {
      "ArrowDown" => FlyKey::LookDown,
      "ArrowLeft" => FlyKey::TurnLeft,
      "ArrowRight" => FlyKey::TurnRight,
      "ArrowUp" => FlyKey::LookUp,
      "KeyA" => FlyKey::Left,
      "KeyD" => FlyKey::Right,
      "KeyE" => FlyKey::Up,
      "KeyQ" => FlyKey::Down,
      "KeyS" => FlyKey::Back,
      "KeyW" => FlyKey::Forward,
      "ShiftLeft" | "ShiftRight" => FlyKey::Fast,
      _ => return None,
    })
  }
}
