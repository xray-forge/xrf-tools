use crate::ai::ai_vertex::AiVertex;

/// The nodes a cover node traces its rays to (`Query::Perform`, `xrAI/compiler_cover.cpp:202`): every node reached
/// through links whose position lies within `cover_distance` of the cover node's, in the order the engine's recursion
/// lists them, the cover node first. A node past the distance is marked and not walked through, as in the engine.
pub(crate) struct AiCoverQuery {
  /// The query each node was last marked by, so a query clears none.
  marks: Vec<u32>,
  query: u32,
  stack: Vec<u32>,
  /// The nodes the last query listed.
  pub list: Vec<u32>,
}

impl AiCoverQuery {
  /// `cover_sqr_dist`, 30 m squared.
  pub(crate) const SQUARED_REACH: f32 = 30.0 * 30.0;

  pub(crate) fn new(node_count: usize) -> Self {
    Self {
      marks: vec![0; node_count],
      query: 0,
      stack: Vec::new(),
      list: Vec::new(),
    }
  }

  /// Lists the nodes the cover node `start` of `vertices` reaches. The engine's recursion visits a node's links in
  /// order, depth first, marking each as it is visited; taking each link from a stack, pushed last to first and marked
  /// as it is taken, visits them in the same order.
  pub(crate) fn perform(&mut self, vertices: &[AiVertex], start: u32) {
    let base = vertices[start as usize].position;

    self.query += 1;
    self.list.clear();
    self.stack.clear();
    self.stack.push(start);

    while let Some(node) = self.stack.pop() {
      let Some(mark) = self.marks.get_mut(node as usize) else {
        continue;
      };

      if *mark == self.query {
        continue;
      }

      *mark = self.query;

      let vertex: &AiVertex = &vertices[node as usize];

      if base.distance_squared(&vertex.position) > Self::SQUARED_REACH {
        continue;
      }

      self.list.push(node);
      self
        .stack
        .extend(vertex.links.iter().rev().filter(|link| **link != AiVertex::NO_LINK));
    }
  }
}
