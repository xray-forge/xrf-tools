use crate::spawn::spawn_ai_map::SpawnAiMap;

/// One node the search has met, `CGraphVertex` of the bucket list's storage.
#[derive(Clone, Copy)]
struct SearchNode {
  /// The search that met it; a node of an older search counts as unvisited.
  search: u32,
  g: f32,
  h: f32,
  f: f32,
  parent: u32,
  is_open: bool,
  bucket: u32,
  previous: u32,
  next: u32,
}

/// xrAI's level path search (`CAStar` over `CLevelGraph` with the straight line path manager, `graph_engine.h:54`):
/// A* from node to node, a step costing a cell's size, the estimate twice the cell size by the cells across and along
/// from the node being expanded to the goal (`path_manager_level_inline.h:94`, which reads the expanded node's cell, not
/// the neighbour's), so not admissible; a closed node is never reopened. The open list is `CDataStorageBucketList` of
/// 8,192 buckets over `f` from 0 to 1,000, each sorted by `f` with a new node ahead of equal ones. Which path it finds
/// decides the edge's weight, so every rule is kept, ties included. Its buffers are kept from search to search.
pub(crate) struct SpawnPathSearch {
  nodes: Vec<SearchNode>,
  buckets: Vec<u32>,
  search: u32,
  minimum_bucket: usize,
}

impl SpawnPathSearch {
  const BUCKETS: usize = 8 * 1024;
  const MAX_BUCKET_VALUE: f32 = 1000.0;
  /// `SStraightLineParams::max_range`: a search whose best `f` reaches it fails.
  pub(crate) const MAX_RANGE: f32 = 6000.0;
  const NONE: u32 = u32::MAX;

  pub(crate) fn new(map: &SpawnAiMap) -> Self {
    Self {
      nodes: vec![
        SearchNode {
          search: 0,
          g: 0.0,
          h: 0.0,
          f: 0.0,
          parent: Self::NONE,
          is_open: false,
          bucket: 0,
          previous: Self::NONE,
          next: Self::NONE,
        };
        map.get_node_count()
      ],
      buckets: vec![Self::NONE; Self::BUCKETS],
      search: 0,
      minimum_bucket: Self::BUCKETS,
    }
  }

  /// The nodes from `start` to `goal` the search walks, both included; `None` where it fails.
  pub(crate) fn find(&mut self, map: &SpawnAiMap, start: u32, goal: u32) -> Option<Vec<u32>> {
    self.search += 1;
    self.buckets.fill(Self::NONE);
    self.minimum_bucket = Self::BUCKETS;

    let cell: f32 = map.get_header().size;
    let goal_cell: (i64, i64) = map.get_node_cell(goal);
    // The estimate of every node the next expansion meets, from the node it expands (`x1`, `z1`).
    let estimate = |from: (i64, i64)| 2.0 * cell * ((goal_cell.0 - from.0).abs() + (goal_cell.1 - from.1).abs()) as f32;
    let first_estimate: f32 = estimate(map.get_node_cell(start));

    self.visit(start, 0.0, first_estimate, Self::NONE);

    while self.advance_minimum() {
      let best: u32 = self.buckets[self.minimum_bucket];

      if self.nodes[best as usize].f >= Self::MAX_RANGE {
        return None;
      }

      if best == goal {
        return Some(self.get_path(goal));
      }

      self.pop_best();

      let best_g: f32 = self.nodes[best as usize].g;
      let h: f32 = estimate(map.get_node_cell(best));

      for neighbour in map.get_neighbours(best) {
        let g: f32 = best_g + cell;
        let node: SearchNode = self.nodes[neighbour as usize];

        if node.search != self.search {
          self.visit(neighbour, g, h, best);
        } else if node.is_open && node.g > g {
          self.unlink(neighbour);

          let node: &mut SearchNode = &mut self.nodes[neighbour as usize];

          node.g = g;
          node.f = g + node.h;
          node.parent = best;
          self.link(neighbour);
        }
      }
    }

    None
  }

  /// Meets a node for the first time and opens it.
  fn visit(&mut self, index: u32, g: f32, h: f32, parent: u32) {
    self.nodes[index as usize] = SearchNode {
      search: self.search,
      g,
      h,
      f: g + h,
      parent,
      is_open: true,
      bucket: 0,
      previous: Self::NONE,
      next: Self::NONE,
    };
    self.link(index);
  }

  /// `compute_bucket_id`: the bucket of a node's `f`.
  fn get_bucket(f: f32) -> usize {
    if f >= Self::MAX_BUCKET_VALUE {
      Self::BUCKETS - 1
    } else if f <= 0.0 {
      0
    } else {
      (Self::BUCKETS as f32 * f / Self::MAX_BUCKET_VALUE) as usize
    }
  }

  /// `add_to_bucket`: into the bucket of its `f`, ahead of the first node there of an `f` as large.
  fn link(&mut self, index: u32) {
    let f: f32 = self.nodes[index as usize].f;
    let bucket: usize = Self::get_bucket(f);

    self.minimum_bucket = self.minimum_bucket.min(bucket);
    self.nodes[index as usize].bucket = bucket as u32;

    let (mut previous, mut next): (u32, u32) = (Self::NONE, self.buckets[bucket]);

    while next != Self::NONE && self.nodes[next as usize].f < f {
      previous = next;
      next = self.nodes[next as usize].next;
    }

    self.nodes[index as usize].previous = previous;
    self.nodes[index as usize].next = next;

    if next != Self::NONE {
      self.nodes[next as usize].previous = index;
    }

    if previous == Self::NONE {
      self.buckets[bucket] = index;
    } else {
      self.nodes[previous as usize].next = index;
    }
  }

  /// Takes a node out of its bucket.
  fn unlink(&mut self, index: u32) {
    let SearchNode {
      previous, next, bucket, ..
    } = self.nodes[index as usize];

    if previous == Self::NONE {
      self.buckets[bucket as usize] = next;
    } else {
      self.nodes[previous as usize].next = next;
    }

    if next != Self::NONE {
      self.nodes[next as usize].previous = previous;
    }
  }

  /// Closes the best node, the first of the lowest bucket.
  fn pop_best(&mut self) {
    let best: u32 = self.buckets[self.minimum_bucket];

    self.unlink(best);
    self.nodes[best as usize].is_open = false;
  }

  /// `is_opened_empty`: moves the lowest bucket up to the first holding a node; false when none does.
  fn advance_minimum(&mut self) -> bool {
    while self.minimum_bucket < Self::BUCKETS && self.buckets[self.minimum_bucket] == Self::NONE {
      self.minimum_bucket += 1;
    }

    self.minimum_bucket < Self::BUCKETS
  }

  /// `get_node_path`: the goal's parents back to the start, in walking order.
  fn get_path(&self, goal: u32) -> Vec<u32> {
    let mut path: Vec<u32> = Vec::new();
    let mut node: u32 = goal;

    while node != Self::NONE {
      path.push(node);
      node = self.nodes[node as usize].parent;
    }

    path.reverse();
    path
  }
}
