use xrf_error::{XrfError, XrfResult};
use xrf_spawn::GraphCrossTableCell;

use crate::spawn::spawn_ai_map::SpawnAiMap;

/// A level's cross table as xrAI builds it (`CGameGraphBuilder::build_cross_table`, `game_graph_builder.cpp:376`): for
/// every AI node, the game vertex whose graph point reaches it first in link steps, and how far.
///
/// Each graph point in turn floods the map breadth first from its node. It takes a node only where it is strictly
/// nearer than the node's owner so far, and goes on only through nodes it takes, so a tie stays with the earlier
/// point; a point always takes its own node. Nodes no graph point reaches keep the first vertex and a distance of
/// `u32::MAX` cells. xrAI keeps a distance row a graph point (zaton: 317 rows of 1.85 M), but only the owner's
/// distance to each node is ever read back, so this keeps that alone: the same tables in a few arrays a node.
pub struct SpawnCrossTable;

impl SpawnCrossTable {
  /// The cells of a level's cross table, game vertices numbered from 0 on the level, for graph points standing on
  /// `nodes` in vertex order.
  ///
  /// # Errors
  ///
  /// Returns an error when there are more graph points than a game vertex id holds, or a node is not one of the map's.
  pub fn build(map: &SpawnAiMap, nodes: &[u32]) -> XrfResult<Vec<GraphCrossTableCell>> {
    let count: usize = map.get_node_count();

    if nodes.len() > usize::from(u16::MAX) + 1 {
      return Err(XrfError::new_invalid_error(format!(
        "A level of {} graph points is past the 65,536 game vertices a cross table names",
        nodes.len()
      )));
    }

    if let Some(node) = nodes.iter().find(|node| **node as usize >= count) {
      return Err(XrfError::new_invalid_error(format!(
        "A graph point stands on node {node} of a map of {count}"
      )));
    }

    // Marked: no graph point reaches the node (`fill_marks`), or it waits in the next fringe.
    let mut marks: Vec<bool> = Self::find_unreached(map, nodes);
    let mut owners: Vec<u16> = vec![0; count];
    let mut owner_distances: Vec<u32> = vec![u32::MAX; count];
    let mut distances: Vec<u32> = vec![u32::MAX; count];
    let mut visited: Vec<u32> = Vec::new();
    let (mut fringe, mut next_fringe): (Vec<u32>, Vec<u32>) = (Vec::new(), Vec::new());

    for (vertex, start) in nodes.iter().enumerate() {
      for node in visited.drain(..) {
        distances[node as usize] = u32::MAX;
      }

      // `recursive_update` forgets the owner's distance at the point's own node first, so the point always takes it.
      owner_distances[*start as usize] = u32::MAX;
      fringe.push(*start);

      let mut distance: u32 = 0;

      while !fringe.is_empty() {
        for node in &fringe {
          let index: usize = *node as usize;

          owners[index] = vertex as u16;
          owner_distances[index] = distance;
          distances[index] = distance;
          visited.push(*node);

          for next in map.get_neighbours(*node) {
            let next_index: usize = next as usize;

            if marks[next_index] || distances[next_index] <= distance || owner_distances[next_index] <= distance + 1 {
              continue;
            }

            next_fringe.push(next);
            marks[next_index] = true;
          }
        }

        for node in &fringe {
          marks[*node as usize] = false;
        }

        fringe.clear();
        std::mem::swap(&mut fringe, &mut next_fringe);
        distance += 1;
      }
    }

    let cell_size: f32 = map.get_header().size;

    Ok(
      owners
        .into_iter()
        .zip(owner_distances)
        .map(|(game_vertex_id, distance)| GraphCrossTableCell {
          game_vertex_id,
          distance: distance as f32 * cell_size,
        })
        .collect(),
    )
  }

  /// Every node no graph point's node reaches over links (`fill_marks`).
  fn find_unreached(map: &SpawnAiMap, nodes: &[u32]) -> Vec<bool> {
    let mut unreached: Vec<bool> = vec![true; map.get_node_count()];
    let mut stack: Vec<u32> = nodes.to_vec();

    while let Some(node) = stack.pop() {
      if !unreached[node as usize] {
        continue;
      }

      unreached[node as usize] = false;
      stack.extend(map.get_neighbours(node).filter(|next| unreached[*next as usize]));
    }

    unreached
  }
}
