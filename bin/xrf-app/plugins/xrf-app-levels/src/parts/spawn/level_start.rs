use serde::Serialize;
use xrf_math::Vector3d;
use xrf_visual::convert_vector;

use crate::parts::LevelStartOrigin;

/// Where a level opens, in renderer space: an actor's eye where the game puts one on the level.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelStart {
  /// Where the camera stands.
  pub position: Vector3d,
  /// Which way it looks, level with the horizon.
  pub direction: Vector3d,
  /// What the place was taken from, which only the log says.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub origin: LevelStartOrigin,
}

impl LevelStart {
  /// An `eye` of the engine's space in the renderer's, turned to `heading` as `CActor::MoveActor` turns it: the forward
  /// axis of `setHP` of the heading.
  pub(crate) fn of(eye: &Vector3d, heading: f32, origin: LevelStartOrigin) -> Self {
    Self {
      direction: convert_vector(&Vector3d::new(-heading.sin(), 0.0, heading.cos())),
      origin,
      position: convert_vector(eye),
    }
  }
}
