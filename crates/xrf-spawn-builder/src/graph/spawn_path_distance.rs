use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::graph::spawn_graph_point::SpawnGraphPoint;
use crate::map::spawn_ai_map::SpawnAiMap;
use crate::map::spawn_path_search::SpawnPathSearch;

/// The length of a level game graph edge, `CGameGraphBuilder::path_distance`.
///
/// It is the flat distance where a straight walk from one graph point's node reaches the other point, else the length
/// of the level path search's path smoothed by straight walks, `CLevelStraightLinePathManager::create_path`.
pub(crate) struct SpawnPathDistance<'a> {
  map: &'a SpawnAiMap,
  search: SpawnPathSearch,
}

impl<'a> SpawnPathDistance<'a> {
  pub(crate) fn new(map: &'a SpawnAiMap) -> Self {
    Self {
      map,
      search: SpawnPathSearch::new(map),
    }
  }

  /// The weight of the edge from graph point `from` to `to`.
  ///
  /// # Errors
  ///
  /// Returns an error where no path joins the two points' nodes, which xrAI asserts on.
  pub(crate) fn measure(&mut self, from: &SpawnGraphPoint, to: &SpawnGraphPoint) -> XrfResult<f32> {
    let (start, finish) = (Self::flatten(&from.position), Self::flatten(&to.position));

    if self.map.find_straight_walk(from.node, start, finish).is_some() {
      let (dx, dz) = (finish.0 - start.0, finish.1 - start.1);

      return Ok((dx * dx + dz * dz).sqrt());
    }

    let path: Vec<u32> = self.search.find(self.map, from.node, to.node).ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "No path on the AI map joins graph point {} to graph point {}",
        from.name, to.name
      ))
    })?;

    Ok(self.smooth(&path, &from.position, &to.position))
  }

  /// `create_path`'s length of `path` from `start` to `finish`, capped at the search's range.
  ///
  /// The walk advances to the last node it reaches straight, or steps node to node where it reaches none.
  fn smooth(&self, path: &[u32], start: &Vector3d<f32>, finish: &Vector3d<f32>) -> f32 {
    let max: f32 = SpawnPathSearch::MAX_RANGE;
    let (mut cumulative, mut last_direct): (f32, f32) = (0.0, 0.0);
    let mut position: Vector3d<f32> = *start;
    let mut node: u32 = path[0];
    let mut index: usize = 1;

    while index < path.len() {
      let target: Vector3d<f32> = self.map.get_node_position(path[index]);
      let direct: f32 = self.measure_direct(node, &position, &target);

      if direct == max {
        if last_direct == 0.0 {
          cumulative += self.map.get_node_position(node).distance(&target);
          node = path[index];
        } else {
          cumulative += last_direct;
          last_direct = 0.0;
          index -= 1;
          node = path[index];
        }

        position = self.map.get_node_position(node);
      } else {
        last_direct = direct;
      }

      if cumulative + last_direct >= max {
        return max;
      }

      index += 1;
    }

    let direct: f32 = self.measure_direct(node, &position, finish);

    if direct == max {
      let last: Vector3d<f32> = self.map.get_node_position(path[path.len() - 1]);

      cumulative + last_direct + finish.distance(&last)
    } else {
      cumulative + direct
    }
  }

  /// The distance from `position` to `target` where a straight walk from `node` reaches it, else the search's range.
  fn measure_direct(&self, node: u32, position: &Vector3d<f32>, target: &Vector3d<f32>) -> f32 {
    match self
      .map
      .find_straight_walk(node, Self::flatten(position), Self::flatten(target))
    {
      Some(_) => position.distance(target),
      None => SpawnPathSearch::MAX_RANGE,
    }
  }

  fn flatten(position: &Vector3d<f32>) -> (f32, f32) {
    (position.x, position.z)
  }
}
