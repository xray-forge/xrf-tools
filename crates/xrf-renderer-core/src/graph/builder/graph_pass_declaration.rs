use crate::graph::access::{GraphBufferAccess, GraphTextureAccess};
use crate::graph::frame_graph::FrameGraph;
use crate::graph::record::{GraphPass, GraphPassWork};
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// What every pass builder gathers before the pass is recorded: its name and the accesses it declares.
pub(crate) struct GraphPassDeclaration {
  pub name: &'static str,
  pub textures: Vec<(GraphTexture, GraphTextureAccess)>,
  pub buffers: Vec<(GraphBuffer, GraphBufferAccess)>,
  pub is_kept: bool,
}

impl GraphPassDeclaration {
  pub fn new(name: &'static str) -> Self {
    Self {
      name,
      textures: Vec::new(),
      buffers: Vec::new(),
      is_kept: false,
    }
  }

  /// Adds the pass to the graph, recording what it does.
  pub fn declare<'a>(self, graph: &mut FrameGraph<'a>, work: GraphPassWork<'a>) {
    let group: usize = graph.groups.len() - 1;

    graph.push_pass(GraphPass {
      name: self.name,
      group,
      textures: self.textures,
      buffers: self.buffers,
      is_kept: self.is_kept,
      work,
    });
  }
}
