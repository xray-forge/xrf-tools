use std::collections::VecDeque;

use xrf_job::parallel::prelude::*;
use xrf_level::LevelAiParts;
use xrf_math::Aabb;
use xrf_spawn::AlifeObject;

use crate::map::spawn_ai_map::SpawnAiMap;
use crate::separator::spawn_restrictor::SpawnRestrictor;
use crate::separator::spawn_separator_cut::SpawnSeparatorCut;

/// xrAI's separator check, `CSpaceRestrictorWrapper::verify_connectivity`, kept to real cuts: a flagged restrictor
/// whose border, the nodes it holds in part, cuts nodes off.
///
/// GSC floods from its first node outside the restrictor, border masked, to 6,000 m, and asserts unless the nodes
/// reached and held make the map. That also fires on a part already apart and on held nodes counted twice. Here the
/// map's parts are found once; each part the border touches is flooded into pieces from every outside node, border
/// masked, with no range. The cut is the outside nodes of every piece but each part's largest.
pub(crate) struct SpawnSeparatorCheck<'a> {
  map: &'a SpawnAiMap,
  /// The part each node belongs to, named by its lowest node ([`LevelAiParts`]).
  parts: Vec<u32>,
}

impl<'a> SpawnSeparatorCheck<'a> {
  /// A node in no piece: on the border, held and unreached from outside, or in an untouched part.
  const NO_PIECE: u32 = u32::MAX;

  pub(crate) fn new(map: &'a SpawnAiMap) -> Self {
    // The map holds exactly as many nodes as its header counts.
    let count: u32 = map.get_header().count;
    let mut parts: LevelAiParts = LevelAiParts::new(count);

    for node in 0..count {
      for neighbour in map.get_neighbours(node) {
        parts.join(node, neighbour);
      }
    }

    Self {
      map,
      parts: parts.into_names(),
    }
  }

  /// The cuts of the flagged restrictors among `objects`, in their order, on level `level`.
  pub(crate) fn find_cuts(&self, level: &str, objects: &[AlifeObject]) -> Vec<SpawnSeparatorCut> {
    let restrictors: Vec<SpawnRestrictor> = objects.iter().filter_map(SpawnRestrictor::from_object).collect();

    restrictors
      .par_iter()
      .filter_map(|restrictor| self.find_cut(level, restrictor))
      .collect()
  }

  fn find_cut(&self, level: &str, restrictor: &SpawnRestrictor) -> Option<SpawnSeparatorCut> {
    let bounds: Aabb = restrictor.get_bounds()?;
    let nodes: u32 = self.map.get_header().count;
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
    // Each flooded node's piece; a piece's size counts only its nodes outside the restrictor.
    let mut pieces: Vec<u32> = vec![Self::NO_PIECE; count];
    let mut piece_sizes: Vec<u32> = Vec::new();
    let mut piece_parts: Vec<u32> = Vec::new();
    let mut queue: VecDeque<u32> = VecDeque::new();
    let mut next_piece: u32 = 0;

    for start in 0..nodes {
      let index: usize = start as usize;

      if is_held[index] || pieces[index] != Self::NO_PIECE || !is_touched(index) {
        continue;
      }

      let piece: u32 = next_piece;
      let mut size: u32 = 0;

      next_piece += 1;
      pieces[index] = piece;
      queue.push_back(start);

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
      piece_parts.push(self.parts[index]);
    }

    // The part's largest piece stays, the first of equals; the rest is cut off.
    let mut kept: Vec<u32> = Vec::with_capacity(touched.len());

    for part in &touched {
      kept.extend(
        (0..next_piece)
          .filter(|piece| piece_parts[*piece as usize] == *part)
          .max_by_key(|piece| (piece_sizes[*piece as usize], std::cmp::Reverse(*piece))),
      );
    }

    let mut cut = (0..nodes).filter(|node| {
      let node: usize = *node as usize;

      !is_held[node] && pieces[node] != Self::NO_PIECE && !kept.contains(&pieces[node])
    });
    let first: u32 = cut.next()?;

    Some(SpawnSeparatorCut {
      level: level.to_owned(),
      restrictor: restrictor.name.clone(),
      nodes: cut.fold(1, |nodes, _| nodes + 1),
      first_node: first,
      first_position: self.map.get_node_position(first),
    })
  }
}
