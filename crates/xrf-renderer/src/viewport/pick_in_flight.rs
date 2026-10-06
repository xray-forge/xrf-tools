use glam::{Mat4, Vec2};

use crate::viewport::pending_pick::PendingPick;

/// A pick drawn and on its way back: the readback it went into, how its depth unprojects, and the frame it was drawn
/// in.
pub struct PickInFlight {
  pub pick: PendingPick,
  pub slot: usize,
  /// The camera's inverse view projection, and the pick's point on the screen in normalised coordinates.
  pub unprojection: (Mat4, Vec2),
  pub frame: u64,
}
