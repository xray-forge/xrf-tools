use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::spawn::spawn_ai_map::SpawnAiMap;
use crate::spawn::spawn_graph_point::SpawnGraphPoint;
use crate::spawn::spawn_path_search::SpawnPathSearch;

/// How far an edge of a level's game graph runs, xrAI's `CGameGraphBuilder::path_distance` (`game_graph_builder.cpp:457`):
/// the distance across the ground where a straight walk from one graph point's node toward the other's point reaches
/// it, else the length of the level path search's path smoothed by straight walks (`CLevelStraightLinePathManager::
/// create_path`, `path_manager_level_straight_line_inline.h:62`).
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

  /// `create_path`'s length of a path from `start` to `finish`: from where it stands, the walk keeps the last node it
  /// sees straight; where it sees the next no more, it goes on from that last node, or, with none, steps node to node.
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
          cumulative += Self::distance(&self.map.get_node_position(node), &target);
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

      cumulative + last_direct + Self::distance(finish, &last)
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
      Some(_) => Self::distance(position, target),
      None => SpawnPathSearch::MAX_RANGE,
    }
  }

  /// `Fvector::distance_to`, summed as the engine sums it.
  fn distance(from: &Vector3d<f32>, to: &Vector3d<f32>) -> f32 {
    let (dx, dy, dz) = (to.x - from.x, to.y - from.y, to.z - from.z);

    (dx * dx + dy * dy + dz * dz).sqrt()
  }

  fn flatten(position: &Vector3d<f32>) -> (f32, f32) {
    (position.x, position.z)
  }
}
