/// What a held key or button asks of movement, whichever key the player bound to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MovementAction {
  Forward,
  Back,
  Left,
  Right,
  Up,
  Down,
  /// Call of Pripyat's `accel`: on foot the actor runs unless it is held, which walks; in flight it boosts.
  Accel,
  /// Call of Pripyat's `sprint_toggle`: each press turns sprinting on or off, as `g_sprint_toggle on` plays it.
  Sprint,
}

impl MovementAction {
  pub const ALL: [Self; 8] = [
    Self::Forward,
    Self::Back,
    Self::Left,
    Self::Right,
    Self::Up,
    Self::Down,
    Self::Accel,
    Self::Sprint,
  ];

  /// Its place in [`Self::ALL`].
  pub(crate) fn get_index(self) -> usize {
    self as usize
  }
}
