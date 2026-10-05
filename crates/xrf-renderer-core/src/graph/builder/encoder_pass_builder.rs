use crate::graph::access::{GraphBufferAccess, GraphTextureAccess};
use crate::graph::builder::graph_pass_declaration::GraphPassDeclaration;
use crate::graph::execute::EncoderContext;
use crate::graph::frame_graph::FrameGraph;
use crate::graph::record::GraphPassWork;
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// Declares an encoder pass: its accesses, then what it records straight into the encoder.
pub struct EncoderPassBuilder<'g, 'a> {
  graph: &'g mut FrameGraph<'a>,
  declaration: GraphPassDeclaration,
  is_bridge: bool,
}

impl<'g, 'a> EncoderPassBuilder<'g, 'a> {
  pub(crate) fn new(graph: &'g mut FrameGraph<'a>, name: &'static str) -> Self {
    Self {
      graph,
      declaration: GraphPassDeclaration::new(name),
      is_bridge: false,
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

  /// Keeps the pass whether or not anything reads what it writes.
  pub fn keep(mut self) -> Self {
    self.declaration.is_kept = true;
    self
  }

  /// Marks it a pass recorded as before the graph, opening its own render and compute passes; its accesses are
  /// declared by hand and the graph cannot check them.
  pub fn bridge(mut self) -> Self {
    self.is_bridge = true;
    self
  }

  /// Adds the pass, recording with `record` into the encode group's encoder.
  pub fn record(self, record: impl for<'c> FnOnce(&mut EncoderContext<'c>) + Send + 'a) {
    self.declaration.declare(
      self.graph,
      GraphPassWork::Encoder {
        is_bridge: self.is_bridge,
        record: Box::new(record),
      },
    );
  }
}
