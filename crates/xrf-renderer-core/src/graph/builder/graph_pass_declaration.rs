use crate::graph::access::{GraphBufferAccess, GraphTextureAccess};
use crate::graph::frame_graph::FrameGraph;
use crate::graph::graph_owner::GraphOwner;
use crate::graph::record::{GraphPass, GraphPassWork};
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// What every pass builder gathers before the pass is recorded: its name, the accesses it declares and its tags.
pub(crate) struct GraphPassDeclaration {
  pub name: &'static str,
  pub textures: Vec<(GraphTexture, GraphTextureAccess)>,
  pub buffers: Vec<(GraphBuffer, GraphBufferAccess)>,
  pub is_kept: bool,
  pub tags: Vec<&'static str>,
}

impl GraphPassDeclaration {
  pub(super) fn new(name: &'static str) -> Self {
    Self {
      name,
      textures: Vec::new(),
      buffers: Vec::new(),
      is_kept: false,
      tags: Vec::new(),
    }
  }

  /// Adds the pass to the graph, recording what it does.
  pub(super) fn declare<'a>(self, graph: &mut FrameGraph<'a>, work: GraphPassWork<'a>) {
    let group: usize = graph.groups.len() - 1;
    let owner: GraphOwner = graph.owner;

    graph.push_pass(GraphPass {
      name: self.name,
      group,
      owner,
      textures: self.textures,
      buffers: self.buffers,
      is_kept: self.is_kept,
      tags: self.tags,
      work,
    });
  }
}
