use std::collections::VecDeque;

use rayon::prelude::*;
use xrf_math::Aabb;
use xrf_spawn::AlifeObject;

use crate::spawn::spawn_ai_map::SpawnAiMap;
use crate::spawn::spawn_restrictor::SpawnRestrictor;
use crate::spawn::spawn_separator_cut::SpawnSeparatorCut;

/// xrAI's separator check (`CSpaceRestrictorWrapper::verify_connectivity`, `space_restrictor_wrapper.cpp:218`), kept to
/// what it means to find: a restrictor flagged for it whose border, the nodes it holds in part but not whole, cuts
/// nodes off.
///
/// GSC floods the whole map from its first node outside the restrictor, the border masked, to 6,000 m, and asserts
/// unless the nodes reached and the nodes held make the map. That also fires where a part of the map was apart before
/// the restrictor, and where nodes held whole are reached around the border, counted twice. Here the map's parts, a
/// link joining either way, are found once; each part the border touches is flooded from every node outside the
/// restrictor, the border masked, with no range, into pieces; and what is cut is the nodes outside the restrictor of
/// every piece but the part's largest, however GSC's first node falls.
pub(crate) struct SpawnSeparatorCheck<'a> {
  map: &'a SpawnAiMap,
  /// The part each node belongs to, named by its root node.
  parts: Vec<u32>,
}

impl<'a> SpawnSeparatorCheck<'a> {
  /// A node in no piece: on the border, held and reached from no node outside, or in a part the border does not
  /// touch.
  const NO_PIECE: u32 = u32::MAX;

  pub(crate) fn new(map: &'a SpawnAiMap) -> Self {
    let count: u32 = map.get_node_count() as u32;
    let mut parents: Vec<u32> = (0..count).collect();

    for node in 0..count {
      for neighbour in map.get_neighbours(node) {
        let (a, b) = (Self::find(&mut parents, node), Self::find(&mut parents, neighbour));

        // The lower root stays, so a part is named the same whatever the order of its links.
        parents[a.max(b) as usize] = a.min(b);
      }
    }

    let parts: Vec<u32> = (0..count).map(|node| Self::find(&mut parents, node)).collect();

    Self { map, parts }
  }

  /// The cuts of the objects' restrictors flagged for the check, in the objects' order, on level `level`.
  pub(crate) fn find_cuts(&self, level: &str, objects: &[AlifeObject]) -> Vec<SpawnSeparatorCut> {
    let restrictors: Vec<SpawnRestrictor> = objects.iter().filter_map(SpawnRestrictor::from_object).collect();

    restrictors
      .par_iter()
      .filter_map(|restrictor| self.find_cut(level, restrictor))
      .collect()
  }

  fn find_cut(&self, level: &str, restrictor: &SpawnRestrictor) -> Option<SpawnSeparatorCut> {
    let bounds: Aabb = restrictor.get_bounds()?;
    let count: usize = self.map.get_node_count();
    let mut is_held: Vec<bool> = vec![false; count];
    let mut is_border: Vec<bool> = vec![false; count];
    let mut touched: Vec<u32> = Vec::new();

    for node in self.map.get_nodes_within(&bounds) {
      let held: u8 = restrictor.count_held_points(self.map, node);

      is_held[node as usize] = held > 0;

      if held > 0 && held < SpawnRestrictor::NODE_POINTS {
        is_border[node as usize] = true;
        touched.push(self.parts[node as usize]);
      }
    }

    if touched.is_empty() {
      return None;
    }

    touched.sort_unstable();
    touched.dedup();

    let is_touched = |node: usize| touched.binary_search(&self.parts[node]).is_ok();
    // Each node of a touched part the flood from a node outside the restrictor reaches, the border masked, labelled
    // by its piece; a piece's size counts its nodes outside the restrictor.
    let mut pieces: Vec<u32> = vec![Self::NO_PIECE; count];
    let mut piece_sizes: Vec<u32> = Vec::new();
    // The part each piece lies in, beside its size.
    let mut piece_parts: Vec<u32> = Vec::new();
    let mut queue: VecDeque<u32> = VecDeque::new();

    for start in 0..count {
      if is_held[start] || pieces[start] != Self::NO_PIECE || !is_touched(start) {
        continue;
      }

      let piece: u32 = piece_sizes.len() as u32;
      let mut size: u32 = 0;

      pieces[start] = piece;
      queue.push_back(start as u32);

      while let Some(node) = queue.pop_front() {
        size += u32::from(!is_held[node as usize]);

        for neighbour in self.map.get_neighbours(node) {
          if pieces[neighbour as usize] == Self::NO_PIECE && !is_border[neighbour as usize] {
            pieces[neighbour as usize] = piece;
            queue.push_back(neighbour);
          }
        }
      }

      piece_sizes.push(size);
      piece_parts.push(self.parts[start]);
    }

    // The part's largest piece stays, the first of equals; the rest is cut off.
    let mut kept: Vec<u32> = Vec::with_capacity(touched.len());

    for part in &touched {
      kept.extend(
        (0..piece_parts.len() as u32)
          .filter(|piece| piece_parts[*piece as usize] == *part)
          .max_by_key(|piece| (piece_sizes[*piece as usize], std::cmp::Reverse(*piece))),
      );
    }

    let mut cut =
      (0..count).filter(|node| !is_held[*node] && pieces[*node] != Self::NO_PIECE && !kept.contains(&pieces[*node]));
    let first: u32 = cut.next()? as u32;

    Some(SpawnSeparatorCut {
      level: level.to_owned(),
      restrictor: restrictor.name.clone(),
      nodes: 1 + cut.count() as u32,
      first_node: first,
      first_position: self.map.get_node_position(first),
    })
  }

  /// The root of `node`'s part, halving each path it walks.
  fn find(parents: &mut [u32], mut node: u32) -> u32 {
    while parents[node as usize] != node {
      parents[node as usize] = parents[parents[node as usize] as usize];
      node = parents[node as usize];
    }

    node
  }
}
