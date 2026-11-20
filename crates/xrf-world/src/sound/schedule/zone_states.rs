use glam::Vec3;

use crate::level::effects::campfire::Campfire;

/// What a level's zones stand in this frame, which their sounds follow: each campfire's switching and where each object
/// motion has its object.
pub(crate) trait ZoneStates {
  /// The campfire spawned as object `id`, as its switch stands this frame.
  fn get_campfire(&mut self, id: u16) -> Campfire;

  /// Where `motion` has its object this frame, in engine space; none while it is read, or where it cannot be, which
  /// leaves the zone where it spawned.
  fn get_place(&mut self, motion: &str) -> Option<Vec3>;
}
