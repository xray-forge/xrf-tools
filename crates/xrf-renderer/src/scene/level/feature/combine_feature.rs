use xrf_renderer_core::{FrameGraph, GraphColorAttachment, GraphTexture, StorageValue};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::combine_parameters::CombineParameters;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The combine: the G-buffer and its light finished into the scene and its high part, with the sky, the haze, the
/// fog, the ambient occlusion, the indirect light, the reflections and the wet surfaces' gloss.
#[derive(Default)]
pub struct CombineFeature;

impl CombineFeature {
  /// Adds the combine into the scene and the high target, reading the indirect light, the reflections, the wet
  /// surface and the wet reach the units before it answered (`products`, in that order).
  pub fn add_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, lit, sky): (ViewTargetHandles, LightingHandles<'a>, SkyParameters<'a>),
    [indirect_light, reflections, wet_surface, wet_reach]: [GraphTexture; 4],
  ) {
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
      haze_map: handles.haze,
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
