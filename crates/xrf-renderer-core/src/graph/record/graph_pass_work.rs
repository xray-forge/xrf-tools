use crate::graph::access::{GraphColorAttachment, GraphDepthAttachment};
use crate::graph::execute::{ComputeContext, EncoderContext, RasterContext};

/// A raster pass's recording, run inside the render pass the graph opens for it.
pub(crate) type RasterRecord<'a> = Box<dyn for<'c> FnOnce(&mut RasterContext<'c>) + Send + 'a>;

/// A compute pass's recording, run inside the compute pass the graph opens for it.
pub(crate) type ComputeRecord<'a> = Box<dyn for<'c> FnOnce(&mut ComputeContext<'c>) + Send + 'a>;

/// An encoder pass's recording, given the encoder itself.
pub(crate) type EncoderRecord<'a> = Box<dyn for<'c> FnOnce(&mut EncoderContext<'c>) + Send + 'a>;

/// What a pass records, and where the graph runs it.
pub(crate) enum GraphPassWork<'a> {
  /// Drawn into its attachments; consecutive raster passes into the same attachments share one render pass.
  Raster {
    colors: Vec<GraphColorAttachment>,
    depth: Option<GraphDepthAttachment>,
    record: RasterRecord<'a>,
  },
  Compute {
    record: ComputeRecord<'a>,
  },
  /// Given the encoder: copies, and, as a bridge, a pass recorded the way it was before the graph, opening its own
  /// render and compute passes.
  Encoder {
    is_bridge: bool,
    record: EncoderRecord<'a>,
  },
}

impl GraphPassWork<'_> {
  pub fn get_colors(&self) -> &[GraphColorAttachment] {
    match self {
      Self::Raster { colors, .. } => colors,
      _ => &[],
    }
  }

  pub fn get_depth(&self) -> Option<&GraphDepthAttachment> {
    match self {
      Self::Raster { depth, .. } => depth.as_ref(),
      _ => None,
    }
  }

  pub fn is_raster(&self) -> bool {
    matches!(self, Self::Raster { .. })
  }
}
