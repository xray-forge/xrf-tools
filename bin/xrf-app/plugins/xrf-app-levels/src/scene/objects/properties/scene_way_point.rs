use serde::Serialize;
use xrf_editable_level::EditableWayPoint;
use xrf_math::Vector3d;

use crate::scene::SceneWayLink;

/// One point of a way, with the links leading from it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneWayPoint {
  pub name: String,
  pub position: Vector3d,
  /// Thirty-two flags the scripts give meaning to.
  pub flags: u32,
  /// Its links in the order the editor keeps them.
  pub links: Vec<SceneWayLink>,
}

impl From<&EditableWayPoint> for SceneWayPoint {
  fn from(point: &EditableWayPoint) -> Self {
    Self {
      name: point.name.clone(),
      position: point.position,
      flags: point.flags,
      links: point.links.iter().map(SceneWayLink::from).collect(),
    }
  }
}
