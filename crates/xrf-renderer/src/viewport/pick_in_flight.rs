use glam::Vec2;

use crate::camera::camera_view::CameraView;
use crate::viewport::pending_pick::PendingPick;

/// A pick drawn and on its way back: the readback it went into, how its depth unprojects, and the frame it was drawn
/// in.
pub struct PickInFlight {
  pub pick: PendingPick,
  pub slot: usize,
  /// The camera the pick looked through, which its depth unprojects by and is measured along, and the pick's point on
  /// the screen in normalised coordinates.
  pub unprojection: (CameraView, Vec2),
  pub frame: u64,
}
