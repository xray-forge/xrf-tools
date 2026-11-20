use std::cmp::Reverse;
use std::collections::BinaryHeap;

use xrf_spawn::{GraphEdge, SpawnGraphsChunk};

/// Where `--start` puts the actor on another level, `CGameSpawnConstructor::process_actor`: the first vertex of that
/// level a game graph search from the actor's vertex reaches, else the level's vertex nearest it in the world.
///
/// The search is `CAStar` with no estimate, so vertices close by edge distance; of equals the most recently opened
/// closes first, as the bucket list orders them. A closed vertex is never reopened.
pub(crate) struct SpawnActorStart;

impl SpawnActorStart {
  /// The vertex the actor at `from` moves to on level `level_id`; `None` when the level has no vertex.
  pub(crate) fn find_vertex(graph: &SpawnGraphsChunk, from: u16, level_id: u8) -> Option<u16> {
    Self::search(graph, from, level_id).or_else(|| Self::find_nearest(graph, from, level_id))
  }

  fn search(graph: &SpawnGraphsChunk, from: u16, level_id: u8) -> Option<u16> {
    let edges: Vec<&[GraphEdge]> = Self::get_edges(graph);
    let mut distances: Vec<Option<f32>> = vec![None; graph.vertices.len()];
    let mut closed: Vec<bool> = vec![false; graph.vertices.len()];
    // By distance, then the most recently opened first; a vertex reached shorter is pushed again.
    let mut open: BinaryHeap<(Reverse<u32>, u32, u16)> = BinaryHeap::new();
    let mut opened: u32 = 0;

    distances[usize::from(from)] = Some(0.0);
    open.push((Reverse(0.0f32.to_bits()), opened, from));

    while let Some((Reverse(bits), _, vertex)) = open.pop() {
      let distance: f32 = f32::from_bits(bits);

      if closed[usize::from(vertex)] || distances[usize::from(vertex)] != Some(distance) {
        continue;
      }

      if graph.vertices[usize::from(vertex)].level_id == level_id {
        return Some(vertex);
      }

      closed[usize::from(vertex)] = true;

      for edge in edges[usize::from(vertex)] {
        let target: usize = usize::from(edge.game_vertex_id);
        let reached: f32 = distance + edge.distance;

        if closed[target] || distances[target].is_some_and(|known| known <= reached) {
          continue;
        }

        distances[target] = Some(reached);
        opened += 1;
        // Distances are never negative, so their bits order as they do.
        open.push((Reverse(reached.to_bits()), opened, edge.game_vertex_id));
      }
    }

    None
  }

  /// The level's vertex whose game point lies nearest the actor's, the first of equals.
  fn find_nearest(graph: &SpawnGraphsChunk, from: u16, level_id: u8) -> Option<u16> {
    let origin = graph.vertices[usize::from(from)].game_point;
    let mut best: Option<(u16, f32)> = None;

    // A game graph numbers its vertices in 16 bits, as the merge that built it checked.
    for (index, vertex) in (0..=u16::MAX).zip(&graph.vertices) {
      if vertex.level_id != level_id {
        continue;
      }

      let (dx, dy, dz) = (
        vertex.game_point.x - origin.x,
        vertex.game_point.y - origin.y,
        vertex.game_point.z - origin.z,
      );
      let distance: f32 = dx * dx + dy * dy + dz * dz;

      if best.is_none_or(|(_, known)| distance < known) {
        best = Some((index, distance));
      }
    }

    best.map(|(index, _)| index)
  }

  /// Each vertex's slice of the graph's edges.
  fn get_edges(graph: &SpawnGraphsChunk) -> Vec<&[GraphEdge]> {
    let mut start: usize = 0;

    graph
      .vertices
      .iter()
      .map(|vertex| {
        let end: usize = start + usize::from(vertex.edges_count);
        let edges: &[GraphEdge] = &graph.edges[start..end];

        start = end;
        edges
      })
      .collect()
  }
}
