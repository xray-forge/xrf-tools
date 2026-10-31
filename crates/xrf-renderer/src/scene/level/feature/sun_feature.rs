use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sun_draw::SunDraw;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::sun_pass::SunPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The sun drawn into the light target over the G-buffer: shadowed by its cascades, or by its trace where that sees it,
/// and by the contact shadows; how much of it each pixel sees, which combine's glint reads; and at the anchored levels
/// its coloured specular into the specular target.
#[derive(Default)]
pub struct SunFeature;

impl SunFeature {
  /// Draws the sun shadowed by the `contact_shadows` and what its trace saw (`sun_traced`), its cascades where none is
  /// traced, filtered `filter_reach` texels each way. Answers how much of the sun each pixel sees.
  pub fn add_passes<'a>(
    &self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (targets, lit, (wet_surface, wet_reach)): (ViewTargetHandles, LightingHandles<'a>, (GraphTexture, GraphTexture)),
    (contact_shadows, sun_traced, filter_reach): (GraphTexture, Option<GraphTexture>, u32),
  ) -> GraphTexture {
    let is_traced: bool = sun_traced.is_some();
    let sun: SunParameters = SunParameters {
      albedo_target: targets.albedo,
      normal_target: targets.normal,
      material_target: targets.material,
      depth_target: targets.depth,
      material_lut: lit.material_lut,
      brdf_table: lit.brdf_table,
      lut_sampler: lit.lut_sampler,
      lighting: lit.lighting,
      shadow_maps: lit.shadow_maps,
      shadows: lit.shadows,
      contact_shadows,
      sun_traced: sun_traced
        .unwrap_or_else(|| bindings.import_view(&mut *graph, "sun untraced", passes.sun.get_untraced())),
      wet_surface,
      wet_reach,
    };
    let (width, height) = targets.size;
    let seen: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "sun seen",
      width,
      height,
      SunPass::SEEN_FORMAT,
    ));

    let is_split: bool = targets.specular.is_some();
    let mut pass = graph
      .add_raster_pass("sun")
      .parameters(&sun)
      .color(GraphColorAttachment::new(
        targets.light,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .color(GraphColorAttachment::new(
        seen,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ));

    if let Some(specular) = targets.specular {
      pass = pass.color(GraphColorAttachment::new(
        specular,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ));
    }

    let draw: SunDraw = SunDraw {
      is_traced,
      is_split,
      filter_reach,
    };

    pass.record(move |context| passes.sun.record(context, view, &sun, draw));

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
