use xrf_error::XrfResult;

use crate::graph::builder::{ComputePassBuilder, EncoderPassBuilder, RasterPassBuilder};
use crate::graph::compile::{CompiledGraph, GraphCompileOptions, GraphCompiler};
use crate::graph::record::{GraphBufferRecord, GraphPass, GraphTextureRecord};
use crate::graph::resource::{GraphBuffer, GraphBufferDescriptor, GraphTexture, GraphTextureDescriptor};

/// One frame's passes and the resources they pass between them, declared in the order they run.
///
/// A transient is made from the pool for the frame; an import is a resource from outside it, bound when the compiled
/// graph executes. A pass with no effect anyone reads is culled, consecutive raster passes into the same attachments
/// share one render pass, and the passes after [`FrameGraph::begin_group`] are encoded on their own.
pub struct FrameGraph<'a> {
  pub(crate) textures: Vec<GraphTextureRecord>,
  pub(crate) buffers: Vec<GraphBufferRecord>,
  pub(crate) passes: Vec<GraphPass<'a>>,
  pub(crate) groups: Vec<&'static str>,
  /// Whose the passes declared now are, which their costs are summed by.
  pub(crate) owner: u32,
}

impl Default for FrameGraph<'_> {
  fn default() -> Self {
    Self::new()
  }
}

impl<'a> FrameGraph<'a> {
  /// The encode group passes fall in before any other begins.
  pub const DEFAULT_GROUP: &'static str = "frame";
  /// The owner of passes declared before any other is named: the frame's own work.
  pub const FRAME_OWNER: u32 = 0;

  pub fn new() -> Self {
    Self {
      textures: Vec::new(),
      buffers: Vec::new(),
      passes: Vec::new(),
      groups: vec![Self::DEFAULT_GROUP],
      owner: Self::FRAME_OWNER,
    }
  }

  /// A texture made for this frame from the pool.
  pub fn create_texture(&mut self, descriptor: GraphTextureDescriptor) -> GraphTexture {
    self.push_texture(descriptor, false)
  }

  /// A texture from outside the frame, bound when the graph executes; writing it is an effect the graph keeps.
  pub fn import_texture(&mut self, descriptor: GraphTextureDescriptor) -> GraphTexture {
    self.push_texture(descriptor, true)
  }

  /// A buffer made for this frame from the pool.
  pub fn create_buffer(&mut self, descriptor: GraphBufferDescriptor) -> GraphBuffer {
    self.push_buffer(descriptor, false)
  }

  /// A buffer from outside the frame, bound when the graph executes; writing it is an effect the graph keeps.
  pub fn import_buffer(&mut self, descriptor: GraphBufferDescriptor) -> GraphBuffer {
    self.push_buffer(descriptor, true)
  }

  /// Starts an encode group: the passes declared after it are recorded into an encoder of their own.
  pub fn begin_group(&mut self, name: &'static str) {
    self.groups.push(name);
  }

  /// Names whose the passes declared after it are, as a view of a frame drawing several; the timer sums each owner's
  /// passes apart (`GraphTimer::take`).
  pub fn begin_owner(&mut self, owner: u32) {
    self.owner = owner;
  }

  pub fn add_raster_pass(&mut self, name: &'static str) -> RasterPassBuilder<'_, 'a> {
    RasterPassBuilder::new(self, name)
  }

  pub fn add_compute_pass(&mut self, name: &'static str) -> ComputePassBuilder<'_, 'a> {
    ComputePassBuilder::new(self, name)
  }

  pub fn add_encoder_pass(&mut self, name: &'static str) -> EncoderPassBuilder<'_, 'a> {
    EncoderPassBuilder::new(self, name)
  }

  /// Culls, orders the lifetimes, pools the transients, merges render passes and groups the encoding.
  ///
  /// # Errors
  ///
  /// Returns an error for a graph that cannot run: a transient read before anything writes it, a raster pass whose
  /// attachments differ in size or that samples its own attachment.
  pub fn compile(self, options: &GraphCompileOptions) -> XrfResult<CompiledGraph<'a>> {
    GraphCompiler::compile(self, options)
  }

  pub(crate) fn push_pass(&mut self, pass: GraphPass<'a>) {
    for (texture, access) in &pass.textures {
      self.textures[texture.get_index()].usage |= access.get_usage();
    }

    for color in pass.work.get_colors() {
      self.textures[color.texture.get_index()].usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }

    if let Some(depth) = pass.work.get_depth() {
      self.textures[depth.texture.get_index()].usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }

    for (buffer, access) in &pass.buffers {
      self.buffers[buffer.get_index()].usage |= access.get_usage();
    }

    self.passes.push(pass);
  }

  fn push_texture(&mut self, descriptor: GraphTextureDescriptor, is_imported: bool) -> GraphTexture {
    self.textures.push(GraphTextureRecord {
      descriptor,
      is_imported,
      usage: wgpu::TextureUsages::empty(),
    });

    GraphTexture::new(self.textures.len() - 1)
  }

  fn push_buffer(&mut self, descriptor: GraphBufferDescriptor, is_imported: bool) -> GraphBuffer {
    self.buffers.push(GraphBufferRecord {
      descriptor,
      is_imported,
      usage: wgpu::BufferUsages::empty(),
    });

    GraphBuffer::new(self.buffers.len() - 1)
  }
}
