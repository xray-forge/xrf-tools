use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, RasterContext, RasterPassBuilder,
};

use crate::pass::grass_card_shadow_parameters::GrassCardShadowParameters;
use crate::pass::grass_pass::GrassPass;
use crate::pass::view_binding::ViewBinding;

/// The grass's cards a frame draws into its near sun cascades, where those shadow it: the pass, what their draw reads
/// and the textures it cuts them by, and their indirect draw as the graph knows it.
#[derive(Clone, Copy)]
pub struct GrassShadowCasters<'a> {
  pub pass: &'a GrassPass,
  pub parameters: GrassCardShadowParameters,
  pub textures: &'a wgpu::BindGroup,
  pub draw: GraphBuffer,
}

impl<'a> GrassShadowCasters<'a> {
  /// Declares on `builder` what the cards' draw reads: its indirect arguments, their corners, their records and the
  /// models.
  pub fn declare<'g>(&self, builder: RasterPassBuilder<'g, 'a>) -> RasterPassBuilder<'g, 'a> {
    builder
      .buffer(self.draw, GraphBufferAccess::Indirect)
      .parameters(&self.parameters)
  }

  /// Draws the cards into the map `context`'s pass draws into, seen by `view`, within the region already set.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding) {
    let draw: &wgpu::Buffer = context.get_buffer(self.draw);

    self
      .pass
      .record_card_shadow(context, view, &self.parameters, self.textures, draw);
  }

  /// Declares the cards' draw named `name` into layer `layer` of the cascades' `maps`, over what they hold, seen by a
  /// cascade's `view`.
  pub fn add_pass(
    &self,
    graph: &mut FrameGraph<'a>,
    name: &'static str,
    (maps, layer): (GraphTexture, u32),
    view: &'a ViewBinding,
  ) {
    let casters: GrassShadowCasters<'a> = *self;

    casters
      .declare(
        graph
          .add_raster_pass(name)
          .depth(GraphDepthAttachment::new(maps, wgpu::LoadOp::Load).with_array_layer(layer)),
      )
      .record(move |context| casters.record(context, view));
  }
}
