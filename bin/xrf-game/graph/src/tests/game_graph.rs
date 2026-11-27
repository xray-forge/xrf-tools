use uuid::Uuid;
use xrf_math::Vector3d;
use xrf_math::glam::Vec3;
use xrf_spawn::{GraphEdge, GraphHeader, GraphLevel, GraphVertex, SpawnGraphsChunk, U32Bytes};

use crate::graph::game_graph::GameGraph;
use crate::id::game_vertex_id::GameVertexId;

/// A vertex on `level` at game point `x` along `+x`, with `edges` edges.
fn vertex(level: u8, x: f32, edges: u8) -> GraphVertex {
  GraphVertex {
    level_point: Vector3d::new(x, 1.0, 0.0),
    game_point: Vector3d::new(x, 0.0, 0.0),
    level_id: level,
    level_vertex_id: 7,
    vertex_type: U32Bytes(1, 2, 3, 4),
    edges_offset: 0,
    level_points_offset: 0,
    edges_count: edges,
    level_points_count: 0,
  }
}

fn edge(to: u16, distance: f32) -> GraphEdge {
  GraphEdge {
    game_vertex_id: to,
    distance,
  }
}

fn level(id: u8, name: &str) -> GraphLevel {
  GraphLevel {
    name: String::from(name),
    offset: Vector3d::new(0.0, 0.0, 0.0),
    id,
    section: format!("level_{name}"),
    guid: Uuid::nil(),
  }
}

/// Three vertices in a line on `zaton`, the middle one joined to both ends, and one on `jupiter` alone.
fn graphs() -> SpawnGraphsChunk {
  let mut graphs: SpawnGraphsChunk = SpawnGraphsChunk {
    header: GraphHeader {
      version: 10,
      vertices_count: 0,
      edges_count: 0,
      points_count: 0,
      guid: Uuid::nil(),
      levels_count: 0,
    },
    levels: vec![level(1, "zaton"), level(2, "jupiter")],
    vertices: vec![
      vertex(1, 0.0, 1),
      vertex(1, 30.0, 2),
      vertex(1, 70.0, 1),
      vertex(2, 900.0, 0),
    ],
    edges: vec![edge(1, 31.5), edge(0, 31.5), edge(2, 42.0), edge(1, 42.0)],
    points: Vec::new(),
    cross_tables: Vec::new(),
  };

  graphs.derive_layout().expect("the test graph lays out");
  graphs
}

#[test]
fn reads_vertices_edges_and_levels_from_the_spawns_graph() {
  let graph: GameGraph = GameGraph::of_spawn(&graphs()).expect("the graph reads");

  assert_eq!(graph.len(), 4);
  assert_eq!(graph.count_edges(), 4);
  assert_eq!(graph.find_level("ZATON").map(|level| level.id), Some(1));
  assert_eq!(
    graph.find_level("jupiter").map(|level| level.section.as_str()),
    Some("level_jupiter")
  );

  let middle = graph.get_vertex(GameVertexId(1)).expect("the middle vertex");

  assert_eq!(middle.level, 1);
  assert_eq!(middle.level_vertex, 7);
  assert_eq!(middle.level_point, Vec3::new(30.0, 1.0, 0.0));
  assert_eq!(middle.kind, [1, 2, 3, 4]);

  let targets = |id: u16| {
    graph
      .get_edges(GameVertexId(id))
      .iter()
      .map(|edge| edge.to.0)
      .collect::<Vec<u16>>()
  };

  assert_eq!(targets(0), vec![1]);
  assert_eq!(targets(1), vec![0, 2]);
  assert_eq!(targets(2), vec![1]);
  assert_eq!(targets(3), Vec::<u16>::new());
  assert_eq!(targets(9), Vec::<u16>::new());
  assert_eq!(graph.get_edges(GameVertexId(1))[1].distance, 42.0);
}

#[test]
fn measures_straight_lines_between_game_points() {
  let graph: GameGraph = GameGraph::of_spawn(&graphs()).expect("the graph reads");

  assert_eq!(graph.get_distance(GameVertexId(0), GameVertexId(2)), Some(70.0));
  assert_eq!(graph.get_distance(GameVertexId(3), GameVertexId(0)), Some(900.0));
  assert_eq!(graph.get_distance(GameVertexId(0), GameVertexId(4)), None);
}

#[test]
fn refuses_edges_off_the_edge_table() {
  let mut graphs: SpawnGraphsChunk = graphs();

  graphs.vertices[2].edges_offset += 1;

  let error: String = GameGraph::of_spawn(&graphs).expect_err("refused").to_string();

  assert!(error.contains("Game vertex 2"), "{error}");
}
