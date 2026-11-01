use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, RasterContext, RasterPassBuilder,
};

use crate::contract::feature::lighting::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::feature::lighting::shadows::cascade_target::CascadeTarget;
use crate::feature::scene::drawable::drawable_kind::DrawableKind;
use crate::feature::scene::drawable::per_frame_casters::PerFrameCasters;
use crate::feature::scene::grass::cards::grass_card_shadow_parameters::GrassCardShadowParameters;
use crate::feature::scene::grass::grass_pass::GrassPass;
use crate::pass::view::view_binding::ViewBinding;

/// The cards' draw into each sun cascade, as the frame's timings name it.
static SUN_PASSES: [&str; RENDER_MAX_SHADOW_CASCADES] = [
  "sun shadow grass 0",
  "sun shadow grass 1",
  "sun shadow grass 2",
  "sun shadow grass 3",
];

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
  /// Metres the grass stands at most over the ground.
  const STANDING: f32 = 3.0;
  /// Milliseconds a cascade's draw of the cards costs before the timer measured one.
  const SUN_SEED: f32 = 0.03;

  /// The cards as the sun's cascades draw them every frame, standing out to `reach` metres from the eye.
  pub fn get_sun_casters(reach: f32) -> PerFrameCasters {
    PerFrameCasters {
      kind: DrawableKind::Grass,
      reach,
      standing: Self::STANDING,
      passes: &SUN_PASSES,
      seed: Self::SUN_SEED,
    }
  }

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

  /// Declares the cards' draw into a cascade of the sun's maps (`target`), over what it holds.
  pub fn add_pass(
    &self,
    graph: &mut FrameGraph<'a>,
    CascadeTarget {
      name,
      maps,
      layer,
      view,
    }: CascadeTarget<'a>,
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
