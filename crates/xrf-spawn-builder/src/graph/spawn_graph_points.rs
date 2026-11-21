use xrf_math::Vector3d;
use xrf_spawn::{AlifeGraphPoint, AlifeObject, AlifeObjectInherited};

use crate::graph::spawn_graph_point::SpawnGraphPoint;
use crate::graph::spawn_graph_point_removal::SpawnGraphPointRemoval;
use crate::graph::spawn_removed_graph_point::SpawnRemovedGraphPoint;
use crate::map::spawn_ai_map::SpawnAiMap;

/// A level's graph points in `level.spawn` order, `load_graph_points`, with the ones left out and why.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpawnGraphPoints {
  pub points: Vec<SpawnGraphPoint>,
  pub removed: Vec<SpawnRemovedGraphPoint>,
}

impl SpawnGraphPoints {
  /// `EPS_L`, the squared distance under which two graph points are one.
  const TOO_CLOSE_SQUARED: f32 = 0.001;

  /// The graph points among a level's spawn list, each placed on the node under it.
  pub fn read(objects: &[AlifeObject], map: &SpawnAiMap) -> Self {
    let mut graph_points: Self = Self::default();

    for object in objects {
      if let AlifeObjectInherited::CseAlifeGraphPoint(point) = &object.inherited {
        graph_points.add(&object.header.name, object.header.position, point, map);
      }
    }

    graph_points
  }

  /// Keeps a graph point, or records why xrAI drops it.
  fn add(&mut self, name: &str, position: Vector3d<f32>, point: &AlifeGraphPoint, map: &SpawnAiMap) {
    match self.place(&position, map) {
      Ok(node) => self.points.push(SpawnGraphPoint {
        name: name.to_owned(),
        position,
        node,
        locations: [point.location0, point.location1, point.location2, point.location3],
        connection_point_name: point.connection_point_name.clone(),
        connection_level_name: point.connection_level_name.clone(),
      }),
      Err(removal) => self.removed.push(SpawnRemovedGraphPoint {
        name: name.to_owned(),
        removal,
      }),
    }
  }

  /// The node a graph point at `position` takes, testing in xrAI's order: not too close to a kept point, over a node,
  /// and not on a kept point's node.
  fn place(&self, position: &Vector3d<f32>, map: &SpawnAiMap) -> Result<u32, SpawnGraphPointRemoval> {
    if self.points.iter().any(|kept| {
      let (dx, dy, dz) = (
        kept.position.x - position.x,
        kept.position.y - position.y,
        kept.position.z - position.z,
      );

      dx * dx + dy * dy + dz * dz < Self::TOO_CLOSE_SQUARED
    }) {
      return Err(SpawnGraphPointRemoval::TooClose);
    }

    let node: u32 = map
      .is_valid_position(position)
      .then(|| map.find_node(position))
      .flatten()
      .ok_or(SpawnGraphPointRemoval::OutsideMap)?;

    if self.points.iter().any(|kept| kept.node == node) {
      return Err(SpawnGraphPointRemoval::SharedNode);
    }

    Ok(node)
  }
}
