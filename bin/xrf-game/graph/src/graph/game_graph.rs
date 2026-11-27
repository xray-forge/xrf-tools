use xrf_error::{XrfError, XrfResult};
use xrf_math::glam::Vec3;
use xrf_spawn::{GraphEdge, GraphVertex, SpawnGraphsChunk};

use crate::graph::game_edge::GameEdge;
use crate::graph::game_graph_level::GameGraphLevel;
use crate::graph::game_vertex::GameVertex;
use crate::id::game_vertex_id::GameVertexId;

/// The game graph (`CGameGraph`, `xrAICore/Navigation/game_graph.h`): a few thousand vertices over every level, joined
/// by edges, the map offline objects walk while the level graph's nodes are only loaded for the level the actor is on.
/// The game's own type, read once from the spawn's graph chunk; it never changes during a game.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameGraph {
  levels: Vec<GameGraphLevel>,
  vertices: Vec<GameVertex>,
  edges: Vec<GameEdge>,
}

impl GameGraph {
  /// The graph a spawn's graph chunk holds; its cross tables, which map a level's nodes to vertices, are not kept.
  ///
  /// # Errors
  ///
  /// Returns an error when a vertex's edges lie off the edge table, or the graph holds more vertices than a sixteen-bit
  /// id names.
  pub fn of_spawn(graphs: &SpawnGraphsChunk) -> XrfResult<Self> {
    if graphs.vertices.len() > usize::from(u16::MAX) {
      return Err(XrfError::new_invalid_error(format!(
        "The game graph holds {} vertices, past the sixteen-bit ids that name them",
        graphs.vertices.len()
      )));
    }

    let mut vertices: Vec<GameVertex> = Vec::with_capacity(graphs.vertices.len());
    let mut edges: Vec<GameEdge> = Vec::with_capacity(graphs.edges.len());

    for (index, vertex) in graphs.vertices.iter().enumerate() {
      let vertex_edges: &[GraphEdge] = graphs
        .get_edges(vertex)
        .ok_or_else(|| XrfError::new_invalid_error(format!("Game vertex {index}'s edges lie off the edge table")))?;
      let first_edge: u32 = u32::try_from(edges.len())
        .map_err(|_| XrfError::new_invalid_error(String::from("The game graph holds more edges than a u32 counts")))?;

      edges.extend(vertex_edges.iter().map(|edge| GameEdge {
        to: GameVertexId(edge.game_vertex_id),
        distance: edge.distance,
      }));
      vertices.push(Self::create_vertex(vertex, first_edge));
    }

    Ok(Self {
      levels: graphs
        .levels
        .iter()
        .map(|level| GameGraphLevel {
          id: level.id,
          name: level.name.clone(),
          section: level.section.clone(),
        })
        .collect(),
      vertices,
      edges,
    })
  }

  pub fn get_levels(&self) -> &[GameGraphLevel] {
    &self.levels
  }

  /// The level named `name`, case aside.
  pub fn find_level(&self, name: &str) -> Option<&GameGraphLevel> {
    self.levels.iter().find(|level| level.name.eq_ignore_ascii_case(name))
  }

  /// How many vertices it holds.
  pub fn len(&self) -> usize {
    self.vertices.len()
  }

  pub fn is_empty(&self) -> bool {
    self.vertices.is_empty()
  }

  /// How many edges it holds, each direction counted.
  pub fn count_edges(&self) -> usize {
    self.edges.len()
  }

  pub fn get_vertex(&self, id: GameVertexId) -> Option<&GameVertex> {
    self.vertices.get(id.get_index())
  }

  /// The edges leaving `id`, none for a vertex it lacks.
  pub fn get_edges(&self, id: GameVertexId) -> &[GameEdge] {
    self.get_vertex(id).map_or(&[], |vertex| {
      let first: usize = vertex.first_edge as usize;

      self
        .edges
        .get(first..first + usize::from(vertex.edge_count))
        .unwrap_or(&[])
    })
  }

  /// The straight-line distance between two vertices' game points, which the simulation weighs targets by
  /// (`utils.graph_distance`); none for a vertex it lacks.
  pub fn get_distance(&self, from: GameVertexId, to: GameVertexId) -> Option<f32> {
    let from: Vec3 = self.get_vertex(from)?.game_point;
    let to: Vec3 = self.get_vertex(to)?.game_point;

    Some(from.distance(to))
  }

  fn create_vertex(vertex: &GraphVertex, first_edge: u32) -> GameVertex {
    let point = |point: &xrf_math::Vector3d<f32>| Vec3::new(point.x, point.y, point.z);

    GameVertex {
      level: vertex.level_id,
      level_vertex: vertex.level_vertex_id,
      level_point: point(&vertex.level_point),
      game_point: point(&vertex.game_point),
      kind: [
        vertex.vertex_type.0,
        vertex.vertex_type.1,
        vertex.vertex_type.2,
        vertex.vertex_type.3,
      ],
      first_edge,
      edge_count: vertex.edges_count,
    }
  }
}
