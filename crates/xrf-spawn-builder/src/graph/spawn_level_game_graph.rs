use std::cmp::Ordering;

use xrf_error::{XrfError, XrfResult};
use xrf_spawn::GraphCrossTableCell;
use xrf_utils::narrow_format_size;

use crate::graph::spawn_game_edge::SpawnGameEdge;
use crate::graph::spawn_graph_point::SpawnGraphPoint;
use crate::graph::spawn_path_distance::SpawnPathDistance;
use crate::map::spawn_ai_map::SpawnAiMap;

/// A level's game graph built from its graph points and cross table, `CGameGraphBuilder::build_graph`: one vertex per
/// graph point.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SpawnLevelGameGraph {
  /// The edges leaving each vertex, in the order xrAI adds them.
  pub edges: Vec<Vec<SpawnGameEdge>>,
}

/// A pair of vertices joined by an edge either way, at the lighter of its two weights (`create_tripples`).
struct SpawnEdgePair {
  weight: f32,
  from: u16,
  to: u16,
}

impl SpawnLevelGameGraph {
  /// Builds the graph: an edge from each vertex to every vertex whose cross table region borders its own
  /// (`fill_neighbours`), weighed by [`SpawnPathDistance`], then the longest edge of each triangle dropped
  /// (`optimize_graph`).
  ///
  /// # Errors
  ///
  /// Returns an error when no path joins two neighbouring vertices, a vertex would have more than 255 edges, or the
  /// level holds more graph points than a game graph numbers.
  pub(crate) fn build(map: &SpawnAiMap, points: &[SpawnGraphPoint], cells: &[GraphCrossTableCell]) -> XrfResult<Self> {
    let mut distance: SpawnPathDistance<'_> = SpawnPathDistance::new(map);
    let mut marks: Vec<u32> = vec![u32::MAX; map.get_node_count()];
    let mut edges: Vec<Vec<SpawnGameEdge>> = Vec::with_capacity(points.len());

    for (vertex, point) in points.iter().enumerate() {
      let vertex: u16 = narrow_format_size(vertex, "A game vertex id")?;
      let neighbours: Vec<u16> = Self::find_neighbours(map, cells, &mut marks, vertex, point.node);

      if neighbours.len() > usize::from(u8::MAX) {
        return Err(XrfError::new_invalid_error(format!(
          "Graph point {} borders {} others, past the 255 edges a game vertex holds",
          point.name,
          neighbours.len()
        )));
      }

      edges.push(
        neighbours
          .into_iter()
          .map(|target| {
            Ok(SpawnGameEdge {
              target,
              distance: distance.measure(point, &points[target as usize])?,
            })
          })
          .collect::<XrfResult<_>>()?,
      );
    }

    let mut graph: Self = Self { edges };

    graph.drop_triangles();

    Ok(graph)
  }

  /// The vertices whose regions border `vertex`'s, in the order a depth first walk of its region from `start` meets
  /// them.
  fn find_neighbours(
    map: &SpawnAiMap,
    cells: &[GraphCrossTableCell],
    marks: &mut [u32],
    vertex: u16,
    start: u32,
  ) -> Vec<u16> {
    let stamp: u32 = u32::from(vertex);
    let mut stack: Vec<u32> = vec![start];
    let mut neighbours: Vec<u16> = Vec::new();

    while let Some(node) = stack.pop() {
      marks[node as usize] = stamp;

      for next in map.get_neighbours(node) {
        if marks[next as usize] == stamp {
          continue;
        }

        let owner: u16 = cells[next as usize].game_vertex_id;

        if owner != vertex {
          if !neighbours.contains(&owner) {
            neighbours.push(owner);
          }

          continue;
        }

        stack.push(next);
      }
    }

    neighbours
  }

  fn find_edge(&self, from: u16, to: u16) -> Option<&SpawnGameEdge> {
    self.edges[from as usize].iter().find(|edge| edge.target == to)
  }

  fn remove_edge(&mut self, from: u16, to: u16) {
    self.edges[from as usize].retain(|edge| edge.target != to);
  }

  /// `optimize_graph`: longest first, each joined pair loses its edges both ways where a third vertex closes a
  /// triangle with it.
  fn drop_triangles(&mut self) {
    let mut pairs: Vec<SpawnEdgePair> = Vec::new();

    // `build` refuses more vertices than 16 bits number, so this numbering loses none.
    for (from, edges) in (0..=u16::MAX).zip(&self.edges) {
      for edge in edges {
        let back: Option<&SpawnGameEdge> = self.find_edge(edge.target, from);

        if edge.target < from && back.is_some() {
          continue;
        }

        pairs.push(SpawnEdgePair {
          weight: back.map_or(edge.distance, |back| edge.distance.min(back.distance)),
          from,
          to: edge.target,
        });
      }
    }

    // todo: xrAI sorts with MSVC's unstable `std::sort`; pairs of equal weight keep their order here, which matches it
    // only while no two weigh the same.
    pairs.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(Ordering::Equal));

    for pair in pairs {
      let closes: bool = self.edges[pair.from as usize].iter().any(|edge| {
        edge.target != pair.to
          && (self.find_edge(pair.to, edge.target).is_some() || self.find_edge(edge.target, pair.to).is_some())
      });

      if closes {
        self.remove_edge(pair.from, pair.to);
        self.remove_edge(pair.to, pair.from);
      }
    }
  }
}
