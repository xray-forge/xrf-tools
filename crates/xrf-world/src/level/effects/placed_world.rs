use xrf_math::glam::Vec3;

use crate::contract::world_toggles::WorldToggles;

/// What placing a level's particles reads of the frame: where the camera stands and the wind blows, in engine space, and
/// what the toggles let play.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlacedWorld<'a> {
  pub eye: Vec3,
  /// The ambient effects' wind, which carries a campfire's idle particles.
  pub wind: Vec3,
  pub toggles: &'a WorldToggles,
}
