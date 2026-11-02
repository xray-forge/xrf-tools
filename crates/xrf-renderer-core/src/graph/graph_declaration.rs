use crate::graph::execute::{GraphBindings, GraphRuntime};
use crate::graph::frame_graph::FrameGraph;

/// What a frame's passes are declared with: the graph they join, the bindings that import the resources they read from
/// outside it, and the runtime their uploads and caches live in.
pub struct GraphDeclaration<'g, 'a> {
  pub graph: &'g mut FrameGraph<'a>,
  pub bindings: &'g mut GraphBindings<'a>,
  pub runtime: &'g mut GraphRuntime,
}

impl<'g, 'a> GraphDeclaration<'g, 'a> {
  pub fn new(
    graph: &'g mut FrameGraph<'a>,
    bindings: &'g mut GraphBindings<'a>,
    runtime: &'g mut GraphRuntime,
  ) -> Self {
    Self {
      graph,
      bindings,
      runtime,
    }
  }
}
