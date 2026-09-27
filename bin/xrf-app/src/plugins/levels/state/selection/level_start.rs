use serde::Serialize;
use xrf_math::Vector3d;

use crate::plugins::levels::state::selection::level_start_origin::LevelStartOrigin;

/// Where a level opens, in renderer space: an actor's eye where the game puts one on the level.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelStart {
  /// Where the camera stands.
  pub position: Vector3d,
  /// Which way it looks, level with the horizon.
  pub direction: Vector3d,
  /// What the place was taken from.
  pub origin: LevelStartOrigin,
}
