use crate::graph::record::GraphPass;
use crate::graph::resource::{GraphBuffer, GraphTexture};

/// A pass that survived compiling: what was declared, the render pass it shares if it is a raster pass, and every
/// resource it names, for the scope its recording reaches through.
pub(crate) struct CompiledPass<'a> {
  pub pass: GraphPass<'a>,
  /// The merged render pass it draws in, by index; `None` for a compute or encoder pass.
  pub render_pass: Option<usize>,
  pub textures: Vec<GraphTexture>,
  pub buffers: Vec<GraphBuffer>,
}
