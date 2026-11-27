use xrf_game_objects::ObjectId;
use xrf_math::glam::Vec3;

/// Where an online object stands after a tick, which presentation draws it at, blended between two ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GamePose {
  pub id: ObjectId,
  /// In the engine's space.
  pub position: Vec3,
  /// As its record holds it.
  pub direction: Vec3,
}
