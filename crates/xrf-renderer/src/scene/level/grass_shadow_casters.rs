use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, RasterContext, RasterPassBuilder,
};

use crate::frame::grass_card_handles::GrassCardHandles;
use crate::pass::grass_pass::GrassPass;
use crate::pass::view_binding::ViewBinding;

/// The grass's cards a frame draws into its near sun cascades, where those shadow it: the pass, what their draw binds
/// and the textures it cuts them by, and the cards as the graph knows them.
#[derive(Clone, Copy)]
pub struct GrassShadowCasters<'a> {
  pub pass: &'a GrassPass,
  pub group: &'a wgpu::BindGroup,
  pub textures: &'a wgpu::BindGroup,
  pub cards: GrassCardHandles,
}

impl<'a> GrassShadowCasters<'a> {
  /// Declares on `builder` what the cards' draw reads: its indirect arguments, their corners and their records.
  pub fn declare<'g>(&self, builder: RasterPassBuilder<'g, 'a>) -> RasterPassBuilder<'g, 'a> {
    builder
      .buffer(self.cards.draw, GraphBufferAccess::Indirect)
      .buffer(self.cards.vertices, GraphBufferAccess::StorageRead)
      .buffer(self.cards.triangles, GraphBufferAccess::StorageRead)
  }

  /// Draws the cards into the map `context`'s pass draws into, seen by `view`, within the region already set.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding) {
    let draw: &wgpu::Buffer = context.get_buffer(self.cards.draw);

    self
      .pass
      .record_card_shadow(context.get_pass(), view, (self.group, self.textures), draw);
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
    let draw: GraphBuffer = casters.cards.draw;

    graph
      .add_raster_pass(name)
      .depth(GraphDepthAttachment::new(maps, wgpu::LoadOp::Load).with_array_layer(layer))
      .buffer(draw, GraphBufferAccess::Indirect)
      .buffer(casters.cards.vertices, GraphBufferAccess::StorageRead)
      .buffer(casters.cards.triangles, GraphBufferAccess::StorageRead)
      .record(move |context| {
        let draw: &wgpu::Buffer = context.get_buffer(draw);

        casters
          .pass
          .record_card_shadow(context.get_pass(), view, (casters.group, casters.textures), draw);
      });
  }
}
