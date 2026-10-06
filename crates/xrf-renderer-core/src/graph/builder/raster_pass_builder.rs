use crate::graph::access::{GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphTextureAccess};
use crate::graph::builder::graph_pass_declaration::GraphPassDeclaration;
use crate::graph::execute::RasterContext;
use crate::graph::frame_graph::FrameGraph;
use crate::graph::record::GraphPassWork;
use crate::graph::resource::{GraphBuffer, GraphTexture};
use crate::param::PassParameters;

/// Declares a raster pass: its attachments and accesses, then what it draws.
pub struct RasterPassBuilder<'g, 'a> {
  graph: &'g mut FrameGraph<'a>,
  declaration: GraphPassDeclaration,
  colors: Vec<GraphColorAttachment>,
  depth: Option<GraphDepthAttachment>,
}

impl<'g, 'a> RasterPassBuilder<'g, 'a> {
  pub(crate) fn new(graph: &'g mut FrameGraph<'a>, name: &'static str) -> Self {
    Self {
      graph,
      declaration: GraphPassDeclaration::new(name),
      colors: Vec::new(),
      depth: None,
    }
  }

  pub fn color(mut self, attachment: GraphColorAttachment) -> Self {
    self.colors.push(attachment);
    self
  }

  pub fn depth(mut self, attachment: GraphDepthAttachment) -> Self {
    self.depth = Some(attachment);
    self
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

  /// Keeps the pass whether or not anything reads what it draws.
  pub fn keep(mut self) -> Self {
    self.declaration.is_kept = true;
    self
  }

  /// Adds the pass, drawing with `record` inside the render pass the graph opens on its attachments.
  pub fn record(self, record: impl for<'c> FnOnce(&mut RasterContext<'c>) + Send + 'a) {
    self.declaration.declare(
      self.graph,
      GraphPassWork::Raster {
        colors: self.colors,
        depth: self.depth,
        record: Box::new(record),
      },
    );
  }
}
