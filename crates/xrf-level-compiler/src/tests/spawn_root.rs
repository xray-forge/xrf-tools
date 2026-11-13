use uuid::{Uuid, uuid};
use xrf_error::XrfResult;
use xrf_level::{LevelGameFile, LevelGameWay, LevelGameWayLink, LevelGameWayPoint};
use xrf_math::Vector3d;
use xrf_spawn::{AlifeObject, AlifeObjectInherited, GraphCrossTableCell, LevelSpawnFile, SpawnFile};

use crate::tests::fixtures::{actor_object, changer_object, graph_point_object, grid, spawn_file, zone_object};
use crate::{
  SpawnAiMap, SpawnCrossTable, SpawnGraphPoints, SpawnLevelGameGraph, SpawnMergeLevel, SpawnRootBuilder, SpawnRootLevel,
};

/// A level's files, and the graph and cross table built from them.
struct Level {
  map: SpawnAiMap,
  list: LevelSpawnFile,
  game: LevelGameFile,
  points: SpawnGraphPoints,
  graph: SpawnLevelGameGraph,
  cells: Vec<GraphCrossTableCell>,
}

impl Level {
  fn new(map: SpawnAiMap, objects: &[AlifeObject], ways: Vec<LevelGameWay>) -> XrfResult<Self> {
    let list: LevelSpawnFile = spawn_file(objects)?;
    let points: SpawnGraphPoints = SpawnGraphPoints::read(&list, &map)?;
    let nodes: Vec<u32> = points.points.iter().map(|point| point.node).collect();
    let cells: Vec<GraphCrossTableCell> = SpawnCrossTable::build(&map, &nodes)?;
    let graph: SpawnLevelGameGraph = SpawnLevelGameGraph::build(&map, &points.points, &cells)?;

    Ok(Self {
      map,
      list,
      game: LevelGameFile {
        rpoints: Vec::new(),
        ways,
      },
      points,
      graph,
      cells,
    })
  }

  fn input(&self, name: &str, id: u8, offset: Vector3d) -> SpawnRootLevel<'_> {
    SpawnRootLevel {
      merge: SpawnMergeLevel {
        section: format!("level{id}"),
        name: name.to_owned(),
        caption: name.to_owned(),
        id,
        offset,
        map: &self.map,
        points: &self.points.points,
        graph: &self.graph,
        cells: &self.cells,
      },
      spawn: &self.list,
      game: Some(&self.game),
    }
  }
}

/// A patrol named `name` through the cells at `points`, each linked to the next.
fn way(name: &str, points: &[(f32, f32)]) -> LevelGameWay {
  LevelGameWay {
    version: 0x13,
    name: name.to_owned(),
    kind: None,
    points: points
      .iter()
      .enumerate()
      .map(|(index, (x, z))| LevelGameWayPoint {
        position: Vector3d::new(*x, 0.0, *z),
        flags: 0,
        name: format!("{name}_{index}"),
      })
      .collect(),
    links: (1..points.len() as u16)
      .map(|to| LevelGameWayLink {
        from: to - 1,
        to,
        probability: 1.0,
      })
      .collect(),
  }
}

/// Level `b` (id 10): six cells by two, cell (5, 1) a hole; graph points at (0, 0), connecting to `a`'s `a_end`, and
/// (5, 0); the actor, a zone and a changer into `a`; a patrol whose second point stands over the hole.
fn level_b(extra: &[AlifeObject], ways: Vec<LevelGameWay>) -> XrfResult<Level> {
  let mut objects: Vec<AlifeObject> = vec![
    graph_point_object("b_start", 0.0, 0.0, "a", "a_end")?,
    graph_point_object("b_end", 5.0, 0.0, "", "")?,
    actor_object("actor", 1.0, 0.0)?,
    zone_object("b_zone", 3.0, 0.0, 1.5, 2)?,
    changer_object("b_to_a", 4.0, 1.0, "a", "a_start")?,
  ];

  objects.extend_from_slice(extra);

  Level::new(
    grid(6, 2, &[(5, 1)], uuid!("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"))?,
    &objects,
    ways,
  )
}

/// Level `a` (id 20, 100 m along x): a row of four cells, graph points at either end, the second connecting to `b`'s
/// `b_start`; a zone, a changer into `b` and a patrol.
fn level_a(extra: &[AlifeObject], ways: Vec<LevelGameWay>) -> XrfResult<Level> {
  let mut objects: Vec<AlifeObject> = vec![
    graph_point_object("a_start", 0.0, 0.0, "", "")?,
    graph_point_object("a_end", 3.0, 0.0, "b", "b_start")?,
    zone_object("a_zone", 1.0, 0.0, 1.0, 1)?,
    changer_object("a_to_b", 2.0, 0.0, "b", "b_start")?,
  ];

  objects.extend_from_slice(extra);

  Level::new(
    grid(4, 1, &[], uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"))?,
    &objects,
    ways,
  )
}

fn levels() -> XrfResult<(Level, Level)> {
  Ok((
    level_a(&[], vec![way("a_walk", &[(2.0, 0.0)])])?,
    level_b(&[], vec![way("b_walk", &[(1.0, 0.0), (5.0, 1.0)])])?,
  ))
}

fn build(a: &Level, b: &Level, start: Option<&str>) -> XrfResult<SpawnFile> {
  Ok(
    SpawnRootBuilder::build(
      &[
        a.input("a", 20, Vector3d::new(100.0, 0.0, 0.0)),
        b.input("b", 10, Vector3d::ZERO),
      ],
      start,
    )?
    .spawn,
  )
}

fn find<'a>(spawn: &'a SpawnFile, name: &str) -> &'a AlifeObject {
  spawn
    .alife_spawn
    .objects
    .iter()
    .find(|object| object.header.name == name)
    .expect("the object")
}

fn assert_refused(a: &Level, b: &Level, start: Option<&str>, message: &str) {
  let error: String = build(a, b, start).expect_err("expect the build refused").to_string();

  assert!(error.contains(message), "Unexpected error: {error}");
}

#[test]
fn lays_the_levels_out_by_id_and_links_their_changers() -> XrfResult {
  let (a, b) = levels()?;
  let spawn: SpawnFile = build(&a, &b, None)?;

  // `b`'s objects first, its id the lower; graph points left out; each numbered by its place.
  assert_eq!(
    spawn
      .alife_spawn
      .objects
      .iter()
      .map(|object| (object.header.name.as_str(), object.header.spawn_id))
      .collect::<Vec<_>>(),
    [("actor", 0), ("b_zone", 1), ("b_to_a", 2), ("a_zone", 3), ("a_to_b", 4)]
  );

  // Vertices 0 and 1 are `b`'s, 2 and 3 `a`'s: each changer takes the vertex at the point it names, and its node.
  let AlifeObjectInherited::SeLevelChanger(into_a) = &find(&spawn, "b_to_a").inherited else {
    panic!("a level changer");
  };

  assert_eq!(
    (
      into_a.dest_game_vertex_id,
      into_a.dest_level_vertex_id,
      into_a.dest_position
    ),
    (2, 0, Vector3d::ZERO)
  );

  let AlifeObjectInherited::SeLevelChanger(into_b) = &find(&spawn, "a_to_b").inherited else {
    panic!("a level changer");
  };

  assert_eq!((into_b.dest_game_vertex_id, into_b.dest_level_vertex_id), (0, 0));

  // The zones' points one level after another, each zone's offset where its own begin.
  let offsets: Vec<Option<u32>> = ["b_zone", "a_zone"]
    .iter()
    .map(|name| find(&spawn, name).get_built_fields().artefact_position_offset)
    .collect();

  assert_eq!(offsets, [Some(0), Some(2)]);
  assert_eq!(spawn.artefact_spawn.nodes.len(), 3);

  // The actor stays where it stands, on `b`'s first vertex, the nearer.
  assert_eq!(find(&spawn, "actor").get_built_fields().game_vertex_id, Some(0));
  assert_eq!(spawn.header.objects_count, 5);
  assert_eq!(spawn.header.levels_count, 2);
  assert_eq!(spawn.header.graph_guid, spawn.graphs.header.guid);

  Ok(())
}

#[test]
fn sorts_patrols_by_name_and_gives_a_point_with_no_node_a_vertex() -> XrfResult {
  let (a, b) = levels()?;
  let spawn: SpawnFile = build(&a, &b, None)?;
  let patrols = &spawn.patrols.patrols;

  assert_eq!(
    patrols.iter().map(|patrol| patrol.name.as_str()).collect::<Vec<_>>(),
    ["a_walk", "b_walk"]
  );
  // A step from `a_end`, two from `a_start`.
  assert_eq!(patrols[0].points[0].game_vertex_id, 3);

  // Over the hole: no node, and the vertex of the nearest node's cell, `b_end`'s.
  let over_hole = &patrols[1].points[1];

  assert_eq!((over_hole.level_vertex_id, over_hole.game_vertex_id), (u32::MAX, 1));
  assert_eq!(patrols[1].links.len(), 1);

  Ok(())
}

#[test]
fn moves_the_actor_only_to_another_start_level() -> XrfResult {
  let (a, b) = levels()?;
  let own: SpawnFile = build(&a, &b, Some("b"))?;

  assert_eq!(find(&own, "actor").header.position, Vector3d::new(1.0, 0.0, 0.0));

  // From `b`'s first vertex the search closes `b_end` at 5 m, then `a_end` across the connection at 103 m.
  let moved: SpawnFile = build(&a, &b, Some("a"))?;
  let actor: &AlifeObject = find(&moved, "actor");

  assert_eq!(actor.get_built_fields().game_vertex_id, Some(3));
  assert_eq!(actor.get_built_fields().level_vertex_id, Some(3));
  assert_eq!(actor.header.position, Vector3d::new(3.0, 0.0, 0.0));

  Ok(())
}

#[test]
fn derives_the_spawn_guid_from_the_spawn() -> XrfResult {
  let (a, b) = levels()?;
  let (first, second) = (build(&a, &b, None)?, build(&a, &b, None)?);

  assert_ne!(first.header.guid, Uuid::nil());
  assert_eq!(first.header.guid, second.header.guid);
  assert_ne!(build(&a, &b, Some("a"))?.header.guid, first.header.guid);

  Ok(())
}

#[test]
fn refuses_a_build_without_one_actor() -> XrfResult {
  let (a, _) = levels()?;
  let lonely: Level = Level::new(
    grid(6, 2, &[(5, 1)], Uuid::nil())?,
    &[
      graph_point_object("b_start", 0.0, 0.0, "a", "a_end")?,
      changer_object("b_to_a", 4.0, 1.0, "a", "a_start")?,
    ],
    Vec::new(),
  )?;

  assert_refused(&a, &lonely, None, "hold no actor");

  let crowded: Level = level_a(&[actor_object("second_actor", 1.0, 0.0)?], Vec::new())?;
  let (_, b) = levels()?;

  assert_refused(&crowded, &b, None, "more than one actor");

  Ok(())
}

#[test]
fn refuses_a_changer_it_cannot_link() -> XrfResult {
  let (a, _) = levels()?;

  let elsewhere: Level = level_b(&[changer_object("b_to_c", 2.0, 0.0, "c", "c_start")?], Vec::new())?;

  assert_refused(&a, &elsewhere, None, "which is not built");

  let missing: Level = level_b(
    &[changer_object("b_to_nowhere", 2.0, 0.0, "a", "a_middle")?],
    Vec::new(),
  )?;

  assert_refused(&a, &missing, None, "which level a lacks");

  Ok(())
}

#[test]
fn refuses_patrols_sharing_a_name_and_an_unknown_start() -> XrfResult {
  let (a, b) = levels()?;
  let twin: Level = level_b(&[], vec![way("a_walk", &[(1.0, 0.0)])])?;

  assert_refused(&a, &twin, None, "Two patrols are named a_walk");
  assert_refused(&a, &b, Some("c"), "is not a level of the build");

  Ok(())
}
