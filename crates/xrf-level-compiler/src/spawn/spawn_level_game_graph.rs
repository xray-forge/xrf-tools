use std::cmp::Ordering;

use xrf_error::{XrfError, XrfResult};
use xrf_spawn::GraphCrossTableCell;

use crate::spawn::spawn_ai_map::SpawnAiMap;
use crate::spawn::spawn_game_edge::SpawnGameEdge;
use crate::spawn::spawn_graph_point::SpawnGraphPoint;
use crate::spawn::spawn_path_distance::SpawnPathDistance;

/// A level's game graph as xrAI builds it from its graph points and cross table (`CGameGraphBuilder::build_graph`,
/// `game_graph_builder.cpp:695`): a vertex a graph point, each with its edges, in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpawnLevelGameGraph {
  /// The edges leaving each vertex, a list a vertex, in the order xrAI adds them.
  pub edges: Vec<Vec<SpawnGameEdge>>,
}

/// A pair of vertices joined by an edge either way, at the lighter of its two weights (`create_tripples`).
struct SpawnEdgePair {
  weight: f32,
  from: u16,
  to: u16,
}

impl SpawnLevelGameGraph {
  /// Builds the graph: each vertex gets an edge to every vertex whose cross table region its own touches, in the order
  /// a walk over its region from its node meets them (`fill_neighbours`), weighed by [`SpawnPathDistance`]; then of
  /// every triangle of edges the longest is dropped both ways, the longest pairs first (`optimize_graph`).
  ///
  /// # Errors
  ///
  /// Returns an error where no path joins two neighbouring vertices, or a vertex would have more than 255 edges.
  pub fn build(map: &SpawnAiMap, points: &[SpawnGraphPoint], cells: &[GraphCrossTableCell]) -> XrfResult<Self> {
    let mut distance: SpawnPathDistance<'_> = SpawnPathDistance::new(map);
    let mut marks: Vec<u32> = vec![u32::MAX; map.get_node_count()];
    let mut edges: Vec<Vec<SpawnGameEdge>> = Vec::with_capacity(points.len());

    for (vertex, point) in points.iter().enumerate() {
      let neighbours: Vec<u16> = Self::find_neighbours(map, cells, &mut marks, vertex as u16, point.node);

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

  /// The vertices whose regions border a vertex's, in the order a depth first walk over its region from its node
  /// meets them, nodes taken from the top of the stack and their links in order.
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

  /// `optimize_graph`: each pair joined either way, longest first, loses its edges where a third vertex closes a
  /// triangle with it, through an edge either way.
  fn drop_triangles(&mut self) {
    let mut pairs: Vec<SpawnEdgePair> = Vec::new();

    for (from, edges) in self.edges.iter().enumerate() {
      let from: u16 = from as u16;

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
