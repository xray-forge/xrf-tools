use crate::graph::access::{GraphBufferAccess, GraphTextureAccess};
use crate::graph::record::graph_pass_work::GraphPassWork;
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// One declared pass: its name, the encode group it falls in and whose it is, what it accesses, and what it records.
pub(crate) struct GraphPass<'a> {
  pub name: &'static str,
  pub group: usize,
  /// Whose it is, which its cost is summed by.
  pub owner: u32,
  pub textures: Vec<(GraphTexture, GraphTextureAccess)>,
  pub buffers: Vec<(GraphBuffer, GraphBufferAccess)>,
  /// Kept whatever reads its outputs: it has an effect the graph cannot see, as a readback or a timer has.
  pub is_kept: bool,
  pub work: GraphPassWork<'a>,
}

impl GraphPass<'_> {
  /// Every texture it reads: those it accesses to read, and attachments it loads rather than clears.
  pub fn list_texture_reads(&self) -> impl Iterator<Item = GraphTexture> + '_ {
    let accessed = self
      .textures
      .iter()
      .filter(|(_, access)| access.is_read())
      .map(|(texture, _)| *texture);
    let colors = self
      .work
      .get_colors()
      .iter()
      .filter(|color| matches!(color.load, wgpu::LoadOp::Load))
      .map(|color| color.texture);
    let depth = self
      .work
      .get_depth()
      .filter(|depth| depth.is_read_only || matches!(depth.load, wgpu::LoadOp::Load))
      .map(|depth| depth.texture);

    accessed.chain(colors).chain(depth)
  }

  /// Every texture it writes: those it accesses to write, and every attachment it draws into.
  pub fn list_texture_writes(&self) -> impl Iterator<Item = GraphTexture> + '_ {
    let accessed = self
      .textures
      .iter()
      .filter(|(_, access)| access.is_write())
      .map(|(texture, _)| *texture);
    let colors = self.work.get_colors().iter().map(|color| color.texture);
    let depth = self
      .work
      .get_depth()
      .filter(|depth| !depth.is_read_only)
      .map(|depth| depth.texture);

    accessed.chain(colors).chain(depth)
  }

  pub fn list_buffer_reads(&self) -> impl Iterator<Item = GraphBuffer> + '_ {
    self
      .buffers
      .iter()
      .filter(|(_, access)| access.is_read())
      .map(|(buffer, _)| *buffer)
  }

  pub fn list_buffer_writes(&self) -> impl Iterator<Item = GraphBuffer> + '_ {
    self
      .buffers
      .iter()
      .filter(|(_, access)| access.is_write())
      .map(|(buffer, _)| *buffer)
  }

  /// Every texture it names, accessed or drawn into.
  pub fn list_textures(&self) -> impl Iterator<Item = GraphTexture> + '_ {
    let colors = self.work.get_colors().iter().map(|color| color.texture);
    let depth = self.work.get_depth().map(|depth| depth.texture);

    self
      .textures
      .iter()
      .map(|(texture, _)| *texture)
      .chain(colors)
      .chain(depth)
  }

  pub fn list_buffers(&self) -> impl Iterator<Item = GraphBuffer> + '_ {
    self.buffers.iter().map(|(buffer, _)| *buffer)
  }
}
