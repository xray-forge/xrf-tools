use serde::Serialize;
use xrf_math::Vector3d;
use xrf_visual::VisualTransform;

use crate::objects::{
  LevelObjectId, LevelObjectKind, LevelObjectLight, LevelObjectPatrol, LevelObjectPatrolPoint, LevelObjectPlacement,
  LevelObjectShape, LevelObjectSpawn, LevelObjectVolume,
};

/// One discrete thing placed in a level: what it is, where it stands, what it belongs to, and what its source records
/// of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObject {
  pub id: LevelObjectId,
  pub kind: LevelObjectKind,
  pub name: String,
  /// Where it stands, its `XFORM` in renderer space; its shapes stand in the space it gives.
  pub transform: VisualTransform,
  /// The object it is a part of, as a patrol point is of its path, listed before it.
  pub parent: Option<LevelObjectId>,
  /// What the spawn records of it, for an object the spawn places.
  pub spawn: Option<LevelObjectSpawn>,
  /// What it covers, for a logic volume.
  pub volume: Option<LevelObjectVolume>,
  /// Its points and their links, for a patrol path.
  pub patrol: Option<LevelObjectPatrol>,
  /// What the spawn records of it, for a patrol point.
  pub patrol_point: Option<LevelObjectPatrolPoint>,
  /// What its companion file records of it, for a placement.
  pub placement: Option<LevelObjectPlacement>,
  /// What the bundle records of it, for a static light.
  pub light: Option<LevelObjectLight>,
}

impl LevelObject {
  /// An object with nothing recorded of it beyond what it is and where it stands.
  pub(crate) fn new(id: LevelObjectId, kind: LevelObjectKind, name: String, transform: VisualTransform) -> Self {
    Self {
      id,
      kind,
      name,
      transform,
      parent: None,
      spawn: None,
      volume: None,
      patrol: None,
      patrol_point: None,
      placement: None,
      light: None,
    }
  }

  /// The space it covers, in its own space: a logic volume's shapes, or those of a placement covering one; `None` for
  /// an object standing at a point or a path, which the helpers and the map draw otherwise.
  pub fn get_shapes(&self) -> Option<&[LevelObjectShape]> {
    match (&self.volume, &self.placement) {
      (Some(volume), _) => Some(&volume.shapes),
      (None, Some(placement)) if !placement.shapes.is_empty() => Some(&placement.shapes),
      _ => None,
    }
  }

  /// A point or vector of engine space in renderer space, mirrored through `z` as an object's transform is.
  pub(crate) fn to_renderer_space(vector: &Vector3d) -> Vector3d {
    Vector3d::new(vector.x, vector.y, -vector.z)
  }
}
