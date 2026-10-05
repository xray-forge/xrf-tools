use crate::graph::access::{GraphBufferAccess, GraphTextureAccess};
use crate::graph::builder::graph_pass_declaration::GraphPassDeclaration;
use crate::graph::execute::ComputeContext;
use crate::graph::frame_graph::FrameGraph;
use crate::graph::record::GraphPassWork;
use crate::graph::resource::{GraphBuffer, GraphTexture};
use crate::param::PassParameters;

/// Declares a compute pass: its accesses, then what it dispatches.
pub struct ComputePassBuilder<'g, 'a> {
  graph: &'g mut FrameGraph<'a>,
  declaration: GraphPassDeclaration,
}

impl<'g, 'a> ComputePassBuilder<'g, 'a> {
  pub(crate) fn new(graph: &'g mut FrameGraph<'a>, name: &'static str) -> Self {
    Self {
      graph,
      declaration: GraphPassDeclaration::new(name),
    }
  }

  pub fn texture(mut self, texture: GraphTexture, access: GraphTextureAccess) -> Self {
    self.declaration.textures.push((texture, access));
    self
  }

  pub fn buffer(mut self, buffer: GraphBuffer, access: GraphBufferAccess) -> Self {
    self.declaration.buffers.push((buffer, access));
    self
  }

  /// Declares what `parameters` read and write, so the pass may bind them while it records.
  pub fn parameters(mut self, parameters: &impl PassParameters) -> Self {
    self.declaration.textures.extend(parameters.list_texture_accesses());
    self.declaration.buffers.extend(parameters.list_buffer_accesses());
    self
  }

  /// Keeps the pass whether or not anything reads what it writes.
  pub fn keep(mut self) -> Self {
    self.declaration.is_kept = true;
    self
  }

  /// Adds the pass, dispatching with `record` inside the compute pass the graph opens.
  pub fn record(self, record: impl for<'c> FnOnce(&mut ComputeContext<'c>) + Send + 'a) {
    self.declaration.declare(
      self.graph,
      GraphPassWork::Compute {
        record: Box::new(record),
      },
    );
  }
}
