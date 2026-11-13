use uuid::Uuid;
use xrf_error::XrfResult;

use crate::tests::fixtures::{grid, point};
use crate::{SpawnAiMap, SpawnCrossTable, SpawnGameEdge, SpawnGraphPoint, SpawnLevelGameGraph};

fn build(map: &SpawnAiMap, points: &[SpawnGraphPoint]) -> XrfResult<SpawnLevelGameGraph> {
  let nodes: Vec<u32> = points.iter().map(|point| point.node).collect();

  SpawnLevelGameGraph::build(map, points, &SpawnCrossTable::build(map, &nodes)?)
}

fn targets(edges: &[SpawnGameEdge]) -> Vec<u16> {
  let mut targets: Vec<u16> = edges.iter().map(|edge| edge.target).collect();

  targets.sort_unstable();
  targets
}

#[test]
fn joins_bordering_regions_and_drops_each_triangles_longest_edge() -> XrfResult {
  let map: SpawnAiMap = grid(6, 6, &[], Uuid::nil())?;
  let points: Vec<SpawnGraphPoint> = vec![point(&map, 0.0, 0.0), point(&map, 5.0, 0.0), point(&map, 0.0, 5.0)];
  let graph: SpawnLevelGameGraph = build(&map, &points)?;

  // The three regions all border each other; the diagonal, the triangle's longest side, goes both ways.
  assert_eq!(targets(&graph.edges[0]), vec![1, 2]);
  assert_eq!(targets(&graph.edges[1]), vec![0]);
  assert_eq!(targets(&graph.edges[2]), vec![0]);

  // Open ground: each edge the straight distance across it.
  for edges in &graph.edges {
    for edge in edges {
      assert_eq!(edge.distance, 5.0);
    }
  }

  Ok(())
}

#[test]
fn weighs_a_walled_edge_by_the_path_around_the_wall() -> XrfResult {
  // A wall along x 2 from z 0 to 3, open only at z 4.
  let map: SpawnAiMap = grid(5, 5, &[(2, 0), (2, 1), (2, 2), (2, 3)], Uuid::nil())?;
  let points: Vec<SpawnGraphPoint> = vec![point(&map, 0.0, 0.0), point(&map, 4.0, 0.0)];
  let graph: SpawnLevelGameGraph = build(&map, &points)?;
  let (there, back) = (graph.edges[0][0], graph.edges[1][0]);

  assert_eq!((there.target, back.target), (1, 0));

  // Longer than the four metres straight through the wall, no longer than the twelve cells around it.
  for edge in [there, back] {
    assert!(
      edge.distance > 4.0 && edge.distance <= 12.0,
      "Unexpected weight {}",
      edge.distance
    );
  }

  Ok(())
}

#[test]
fn leaves_regions_no_link_joins_without_an_edge() -> XrfResult {
  // A wall across the whole map: the two halves' regions never border, so no edge joins them.
  let map: SpawnAiMap = grid(5, 3, &[(2, 0), (2, 1), (2, 2)], Uuid::nil())?;
  let points: Vec<SpawnGraphPoint> = vec![point(&map, 0.0, 0.0), point(&map, 4.0, 0.0)];
  let graph: SpawnLevelGameGraph = build(&map, &points)?;

  assert!(graph.edges.iter().all(Vec::is_empty));

  Ok(())
}
