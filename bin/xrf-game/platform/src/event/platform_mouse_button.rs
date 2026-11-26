#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PlatformMouseButton {
  Left,
  Middle,
  Right,
  /// The first side button, usually back.
  Back,
  /// The second side button, usually forward.
  Forward,
}
