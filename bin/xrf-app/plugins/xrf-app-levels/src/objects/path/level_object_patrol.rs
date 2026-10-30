use serde::Serialize;
use xrf_math::Vector3d;
use xrf_spawn::Patrol;

use crate::objects::{LevelObject, LevelPatrolLink};

/// What a patrol path is drawn and described by: its points where they stand and the links between them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectPatrol {
  /// Every point where it stands, renderer space, by index along the path.
  pub points: Vec<Vector3d>,
  /// The links between them, those reaching past its points left out.
  pub links: Vec<LevelPatrolLink>,
}

impl From<&Patrol> for LevelObjectPatrol {
  fn from(patrol: &Patrol) -> Self {
    let count: u32 = patrol.points.len() as u32;

    Self {
      points: patrol
        .points
        .iter()
        .map(|point| LevelObject::to_renderer_space(&point.position))
        .collect(),
      links: patrol
        .links
        .iter()
        .flat_map(|link| {
          link.links.iter().map(move |(to, weight)| LevelPatrolLink {
            from: link.index,
            to: *to,
            weight: *weight,
          })
        })
        .filter(|link| link.from < count && link.to < count)
        .collect(),
    }
  }
}
