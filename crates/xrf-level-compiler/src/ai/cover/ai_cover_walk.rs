use crate::ai::ai_cell_pick::AiCellPick;
use crate::ai::ai_vertex::AiVertex;

/// Whether a node reaches a cover node by walking straight toward it across the map (`vertex_in_direction`,
/// `xrAI/compiler_cover.cpp:397`): from the node, each step takes the first link whose cell the line toward the cover
/// node crosses and whose cell's middle comes no farther from it, never stepping straight back.
pub(crate) struct AiCoverWalk;

impl AiCoverWalk {
  /// Whether the walk from `start` toward `target` of `vertices` reaches it, the cells `half_cell` across from their
  /// middles. A walk going round in cells equally far from the target, where the engine would never return, ends
  /// unreached once it has taken as many steps as there are nodes.
  pub(crate) fn is_reaching(vertices: &[AiVertex], start: u32, target: u32, half_cell: f32) -> bool {
    let from: (f32, f32) = Self::flatten(&vertices[start as usize]);
    let to: (f32, f32) = Self::flatten(&vertices[target as usize]);
    let direction: (f32, f32) = (to.0 - from.0, to.1 - from.1);
    let mut current: u32 = start;
    let mut previous: u32 = u32::MAX;
    let mut squared_distance: f32 = (from.0 - to.0) * (from.0 - to.0) + (from.1 - to.1) * (from.1 - to.1);

    for _ in 0..vertices.len() {
      let mut next_step: Option<(u32, f32)> = None;

      for next in vertices[current as usize].links {
        if next == previous || next == AiVertex::NO_LINK {
          continue;
        }

        let at: (f32, f32) = Self::flatten(&vertices[next as usize]);
        let min: (f32, f32) = (at.0 - half_cell, at.1 - half_cell);
        let max: (f32, f32) = (at.0 + half_cell, at.1 + half_cell);

        if !AiCellPick::is_crossing(min, max, from, direction) {
          continue;
        }

        if next == target {
          return true;
        }

        #[expect(
          clippy::manual_midpoint,
          reason = "the engine sums in single precision, where `f32::midpoint` sums wider"
        )]
        let middle: (f32, f32) = ((min.0 + max.0) * 0.5, (min.1 + max.1) * 0.5);
        let distance: f32 = (middle.0 - to.0) * (middle.0 - to.0) + (middle.1 - to.1) * (middle.1 - to.1);

        if distance > squared_distance {
          continue;
        }

        next_step = Some((next, distance));

        break;
      }

      let Some((next, distance)) = next_step else {
        return false;
      };

      squared_distance = distance;
      previous = current;
      current = next;
    }

    false
  }

  fn flatten(vertex: &AiVertex) -> (f32, f32) {
    (vertex.position.x, vertex.position.z)
  }
}
