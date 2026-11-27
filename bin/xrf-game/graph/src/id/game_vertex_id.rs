/// A game graph vertex's id, X-Ray's `GameGraph::_GRAPH_ID` (`xrAICore/Navigation/game_graph_space.h`): sixteen bits,
/// the vertex's place in the graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameVertexId(pub u16);

impl GameVertexId {
  /// Its place in the graph's vertices.
  pub(crate) fn get_index(self) -> usize {
    usize::from(self.0)
  }
}
