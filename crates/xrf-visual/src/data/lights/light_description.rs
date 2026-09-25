use serde::Serialize;
use xrf_math::Vector3d;

use crate::data::lights::light_kind::LightKind;

/// One light of a level, in renderer space, as the engine would light with it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightDescription {
  /// The spawned object it belongs to, or the level file's record for one of the level's own.
  pub name: String,
  pub kind: LightKind,
  pub position: Vector3d,
  /// Where a spot points: its bone's third axis.
  pub direction: Vector3d,
  /// Its bone's first axis, which turns a spot's projector about its direction.
  pub right: Vector3d,
  /// Raw, times the lamp's brightness, as the engine hands it to the shaders.
  pub color: [f32; 3],
  pub range: f32,
  /// How far the range strays each frame, either way, at random: a zone's `idle_light_range_delta`.
  pub range_jitter: f32,
  /// A spot's whole cone, in radians.
  pub cone: f32,
  /// Where a spot's projection starts: the lamp's virtual size.
  pub near: f32,
  /// A spot's projector, by its index among the description's projectors.
  pub projector: Option<u32>,
  /// The animation replacing its colour, by its index among the description's animators.
  pub animator: Option<u32>,
  /// What an animated colour, each channel in `[0, 255]`, is multiplied by.
  pub animator_scale: f32,
  /// Whether it casts shadows, which also has it fade and drop out with distance as the engine's shadowed lights do.
  pub is_shadowed: bool,
  /// Whether it is one of the level file's own lights, which the engine draws only with `r2_allow_r1_lights`.
  pub is_level: bool,
}
