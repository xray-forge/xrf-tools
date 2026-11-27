/// What a held key or button asks of movement, whichever key the player bound to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MovementAction {
  Forward,
  Back,
  Left,
  Right,
  Up,
  Down,
  Fast,
}

impl MovementAction {
  pub const ALL: [Self; 7] = [
    Self::Forward,
    Self::Back,
    Self::Left,
    Self::Right,
    Self::Up,
    Self::Down,
    Self::Fast,
  ];

  /// Its place in [`Self::ALL`].
  pub(crate) fn get_index(self) -> usize {
    self as usize
  }
}
