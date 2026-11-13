use uuid::{Uuid, uuid};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_spawn::{GraphCrossTableCell, GraphEdge};

use crate::tests::fixtures::{grid, point};
use crate::{
  SpawnAiMap, SpawnCrossTable, SpawnGameGraphMerge, SpawnGameGraphMerger, SpawnGraphPoint, SpawnLevelGameGraph,
  SpawnMergeLevel, SpawnRandom, SpawnSkippedConnection,
};

/// A level's map, graph points connected as given, and the graph and cross table built from them.
struct Level {
  map: SpawnAiMap,
  points: Vec<SpawnGraphPoint>,
  graph: SpawnLevelGameGraph,
  cells: Vec<GraphCrossTableCell>,
}

impl Level {
  /// A row of `columns` cells with a graph point at each x of `points`, connecting to the level and point named.
  fn new(columns: u32, guid: Uuid, points: &[(f32, &str, &str)]) -> XrfResult<Self> {
    let map: SpawnAiMap = grid(columns, 1, &[], guid)?;
    let points: Vec<SpawnGraphPoint> = points
      .iter()
      .map(|(x, level, target)| SpawnGraphPoint {
        connection_level_name: (*level).to_owned(),
        connection_point_name: (*target).to_owned(),
        ..point(&map, *x, 0.0)
      })
      .collect();
    let nodes: Vec<u32> = points.iter().map(|point| point.node).collect();
    let cells: Vec<GraphCrossTableCell> = SpawnCrossTable::build(&map, &nodes)?;
    let graph: SpawnLevelGameGraph = SpawnLevelGameGraph::build(&map, &points, &cells)?;

    Ok(Self {
      map,
      points,
      graph,
      cells,
    })
  }

  fn input(&self, name: &str, id: u8, offset: Vector3d) -> SpawnMergeLevel<'_> {
    SpawnMergeLevel {
      section: format!("level{id}"),
      name: name.to_owned(),
      caption: name.to_owned(),
      id,
      offset,
      map: &self.map,
      points: &self.points,
      graph: &self.graph,
      cells: &self.cells,
    }
  }
}

/// Level `a` (id 20, 100 m along x) with points at x 0 and 3, the second connecting to `b`'s first; level `b` (id 10)
/// with points at x 0, connecting to `a`'s second, and 2, connecting to a level outside the build.
fn levels() -> XrfResult<(Level, Level)> {
  Ok((
    Level::new(
      4,
      uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"),
      &[(0.0, "", ""), (3.0, "b", "point_0_0")],
    )?,
    Level::new(
      3,
      uuid!("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"),
      &[(0.0, "a", "point_3_0"), (2.0, "c", "point_0_0")],
    )?,
  ))
}

fn merge(a: &Level, b: &Level) -> XrfResult<SpawnGameGraphMerge> {
  SpawnGameGraphMerger::merge(
    &[
      a.input("a", 20, Vector3d::new(100.0, 0.0, 0.0)),
      b.input("b", 10, Vector3d::new(0.0, 0.0, 0.0)),
    ],
    &mut SpawnRandom::new(),
  )
}

#[test]
fn merges_the_levels_by_id_with_their_connections() -> XrfResult {
  let (a, b) = levels()?;
  let merged: SpawnGameGraphMerge = merge(&a, &b)?;
  let graph = &merged.graph;

  // `b` first, its id the lower: its two vertices, then `a`'s.
  assert_eq!(
    graph.levels.iter().map(|level| level.name.as_str()).collect::<Vec<_>>(),
    ["b", "a"]
  );
  assert_eq!(
    graph.vertices.iter().map(|vertex| vertex.level_id).collect::<Vec<_>>(),
    [10, 10, 20, 20]
  );
  assert_eq!(graph.vertices[3].game_point, Vector3d::new(103.0, 0.0, 0.0));

  // `b`'s first point: its edge on the level, then the connection to `a`'s second, 103 m apart.
  let edges = |vertex: usize| {
    let first: usize = graph.vertices[..vertex]
      .iter()
      .map(|it| usize::from(it.edges_count))
      .sum();

    graph.edges[first..][..usize::from(graph.vertices[vertex].edges_count)].to_vec()
  };

  assert_eq!(
    edges(0),
    [
      GraphEdge {
        game_vertex_id: 1,
        distance: 2.0,
      },
      GraphEdge {
        game_vertex_id: 3,
        distance: 103.0,
      },
    ]
  );
  assert_eq!(
    edges(3),
    [
      GraphEdge {
        game_vertex_id: 2,
        distance: 3.0,
      },
      GraphEdge {
        game_vertex_id: 0,
        distance: 103.0,
      },
    ]
  );
  assert_eq!(
    merged.skipped,
    [SpawnSkippedConnection {
      level: String::from("b"),
      point: String::from("point_2_0"),
      target_level: String::from("c"),
    }]
  );

  // `a`'s cross table names its vertices past `b`'s two; every node keeps a level point, ten or fewer a vertex.
  assert!(graph.cross_tables[1].cells.iter().all(|cell| cell.game_vertex_id >= 2));
  assert_eq!(graph.points.len(), 7);
  graph.assert_engine_layout()?;

  Ok(())
}

#[test]
fn derives_the_graph_guid_from_the_graph() -> XrfResult {
  let (a, b) = levels()?;
  let (first, second) = (merge(&a, &b)?, merge(&a, &b)?);

  assert_ne!(first.graph.header.guid, Uuid::nil());
  assert_eq!(first.graph.header.guid, second.graph.header.guid);
  assert!(
    first
      .graph
      .cross_tables
      .iter()
      .all(|table| table.game_guid == first.graph.header.guid)
  );

  let (moved, _) = levels()?;
  let other: SpawnGameGraphMerge = SpawnGameGraphMerger::merge(
    &[
      moved.input("a", 20, Vector3d::new(200.0, 0.0, 0.0)),
      b.input("b", 10, Vector3d::new(0.0, 0.0, 0.0)),
    ],
    &mut SpawnRandom::new(),
  )?;

  assert_ne!(other.graph.header.guid, first.graph.header.guid);

  Ok(())
}

#[test]
fn refuses_a_connection_to_a_point_its_level_lacks() -> XrfResult {
  let a: Level = Level::new(4, Uuid::nil(), &[(0.0, "b", "missing")])?;
  let (_, b) = levels()?;
  let error: String = SpawnGameGraphMerger::merge(
    &[
      a.input("a", 20, Vector3d::new(100.0, 0.0, 0.0)),
      b.input("b", 10, Vector3d::new(0.0, 0.0, 0.0)),
    ],
    &mut SpawnRandom::new(),
  )
  .expect_err("expect the missing graph point refused")
  .to_string();

  assert!(error.contains("which that level lacks"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn refuses_levels_it_cannot_number() -> XrfResult {
  let (a, b) = levels()?;
  let origin: Vector3d = Vector3d::new(0.0, 0.0, 0.0);
  let error = |inputs: &[SpawnMergeLevel<'_>]| {
    SpawnGameGraphMerger::merge(inputs, &mut SpawnRandom::new())
      .expect_err("expect the levels refused")
      .to_string()
  };

  // Ids 10, 15 and 20: the two named alike are not neighbours by id.
  let shared: String = error(&[
    a.input("a", 20, origin),
    b.input("b", 15, origin),
    a.input("a", 10, origin),
  ]);

  assert!(shared.contains("share an id or a name"), "Unexpected error: {shared}");

  let mut short = b.input("b", 10, origin);

  short.cells = &b.cells[1..];

  let mismatched: String = error(&[a.input("a", 20, origin), short]);

  assert!(
    mismatched.contains("was not built from"),
    "Unexpected error: {mismatched}"
  );

  Ok(())
}

#[test]
fn draws_the_msvc_runtime_stream() {
  let mut random: SpawnRandom = SpawnRandom::new();
  let mut items: Vec<u32> = (0..5).collect();

  assert_eq!([random.draw(), random.draw(), random.draw()], [41, 18467, 6334]);

  random.shuffle(&mut items);

  let mut sorted: Vec<u32> = items.clone();

  sorted.sort_unstable();
  assert_eq!(sorted, [0, 1, 2, 3, 4]);
}
