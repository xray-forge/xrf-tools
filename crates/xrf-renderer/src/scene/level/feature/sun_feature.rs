use xrf_renderer_core::{FrameGraph, GraphBindings, GraphColorAttachment, GraphTexture, GraphTextureDescriptor};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::sun_pass::SunPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The sun drawn into the light target over the G-buffer: shadowed by its cascades, or by its trace where that sees it,
/// and by the contact shadows; and how much of it each pixel sees, which combine's glint reads.
#[derive(Default)]
pub struct SunFeature;

impl SunFeature {
  /// Draws the sun shadowed by the `contact_shadows` and what its trace saw (`sun_traced`), its cascades where none is
  /// traced. Answers how much of the sun each pixel sees.
  pub fn add_passes<'a>(
    &self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (targets, lit): (ViewTargetHandles, LightingHandles<'a>),
    (contact_shadows, sun_traced): (GraphTexture, Option<GraphTexture>),
  ) -> GraphTexture {
    let is_traced: bool = sun_traced.is_some();
    let sun: SunParameters = SunParameters {
      normal_target: targets.normal,
      material_target: targets.material,
      depth_target: targets.depth,
      material_lut: lit.material_lut,
      lut_sampler: lit.lut_sampler,
      lighting: lit.lighting,
      shadow_maps: lit.shadow_maps,
      shadows: lit.shadows,
      contact_shadows,
      sun_traced: sun_traced
        .unwrap_or_else(|| bindings.import_view(&mut *graph, "sun untraced", passes.sun.get_untraced())),
    };
    let (width, height) = targets.size;
    let seen: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "sun seen",
      width,
      height,
      SunPass::SEEN_FORMAT,
    ));

    graph
      .add_raster_pass("sun")
      .parameters(&sun)
      .color(GraphColorAttachment::new(
        targets.light,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .color(GraphColorAttachment::new(
        seen,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .record(move |context| passes.sun.record(context, view, &sun, is_traced));

    seen
  }
}

impl ViewFeature for SunFeature {
  /// Nothing between frames: its targets are the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}
