use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphTexture, GraphTextureDescriptor, StorageValue,
};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::combine_parameters::CombineParameters;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::sky_behind_parameters::SkyBehindParameters;
use crate::pass::sky_behind_pass::SkyBehindPass;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The fog's square below which the sky behind changes no pixel by half an eight-bit step, as `frame/combine.wgsl`
/// declares it.
const UNSEEN_SKY: f32 = 1.0 / 512.0;

/// The combine: the G-buffer and its light finished into the scene and its high part, with the sky, the haze, the
/// fog, the ambient occlusion, the indirect light, the reflections and the wet surfaces' gloss.
#[derive(Default)]
pub struct CombineFeature {
  /// Whether the sky behind the fog can show this frame: the sky is drawn, and the fog at the far plane reaches it.
  is_sky_behind: bool,
}

impl CombineFeature {
  /// Takes this frame's `lighting`, through a lens reaching `far` metres.
  pub fn prepare(&mut self, lighting: &LightingUniform, far: f32) {
    let fog: f32 = (far * lighting.fog.y + lighting.fog.x).clamp(0.0, 1.0) * lighting.fog_color.w;

    self.is_sky_behind = lighting.params.y > 0.5 && lighting.fog.w > 0.5 && fog * fog >= UNSEEN_SKY;
  }

  /// Adds the combine into the scene and the high target, reading the indirect light, the reflections, the wet
  /// surface and the wet reach the units before it answered (`products`, in that order).
  pub fn add_passes<'a>(
    &self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, lit, sky): (ViewTargetHandles, LightingHandles<'a>, SkyParameters<'a>),
    [indirect_light, reflections, wet_surface, wet_reach]: [GraphTexture; 4],
  ) {
    let sky_behind_map: GraphTexture = if self.is_sky_behind {
      let (width, height) = handles.size;
      let target: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        "sky behind",
        width.div_ceil(2),
        height.div_ceil(2),
        SkyBehindPass::FORMAT,
      ));
      let parameters: SkyBehindParameters = SkyBehindParameters {
        lighting: lit.lighting,
        exposure: StorageValue::new(lit.exposure),
        haze_map: handles.haze,
        depth_target: handles.depth,
      };

      graph
        .add_raster_pass("sky behind")
        .parameters(&parameters)
        .parameters(&sky)
        .color(GraphColorAttachment::new(
          target,
          wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        ))
        .record(move |context| passes.sky_behind.record(context, view, (&parameters, &sky)));

      target
    } else {
      bindings.import_view(graph, "sky behind unseen", passes.sky_behind.get_unseen())
    };
    let parameters: CombineParameters = CombineParameters {
      albedo_target: handles.albedo,
      normal_target: handles.normal,
      material_target: handles.material,
      depth_target: handles.depth,
      light_target: handles.light,
      material_lut: lit.material_lut,
      lut_sampler: lit.lut_sampler,
      lighting: lit.lighting,
      exposure: StorageValue::new(lit.exposure),
      occlusion_target: handles.occlusion[0],
      sky_behind_map,
      indirect_light,
      reflections,
      wet_surface,
      wet_reach,
    };

    graph
      .add_raster_pass("combine")
      .parameters(&parameters)
      .parameters(&sky)
      .color(GraphColorAttachment::new(
        handles.scene,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .color(GraphColorAttachment::new(
        handles.high,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .record(move |context| passes.combine.record(context, view, &parameters, &sky));
  }
}

impl ViewFeature for CombineFeature {
  /// Its targets are the view's scene and high part.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}
