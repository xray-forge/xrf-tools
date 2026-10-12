use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, StorageValue,
  UniformBinding,
};

use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::froxel_uniform::FroxelUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sun_shafts_parameters::SunShaftsParameters;
use crate::pass::sun_shafts_pass::SunShaftsPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The sun's shafts, marched at half size through its shadow while the weather shafts it, and the composite that adds
/// them, the shadowed lights' shafts or the enhanced volume's froxels in their place over the scene and its high part.
#[derive(Default)]
pub struct SunShaftsFeature {
  /// Whether the sun's shafts are marched this frame: a lit view showing them, the weather shafting the sun, and its
  /// cascades drawn.
  is_shafted: bool,
}

impl SunShaftsFeature {
  /// Takes this frame's shafts.
  pub fn prepare(&mut self, options: &RenderViewOptions, lighting: &RenderLighting) {
    self.is_shafted = options.mode.is_lit
      && options.show.is_sun_shafted
      && lighting.get_sun_shafts(&options.features.sun_shafts) > 0.0
      && options.features.shadows.get_cascade_count() > 0;
  }

  /// Adds the sun's shafts marched, unless the froxels draw in their place (`is_froxeled`), and the composite of them
  /// with the shadowed lights' shafts or the froxels; nothing where none of them is drawn.
  pub fn add_passes<'a>(
    &self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, lit): (ViewTargetHandles, LightingHandles<'a>),
    (light_volumes, is_froxeled): (Option<GraphTexture>, bool),
    froxels: Option<(UniformBinding<FroxelUniform>, GraphTexture)>,
  ) {
    let is_shafted: bool = self.is_shafted && !is_froxeled;

    if !(is_shafted || light_volumes.is_some() || froxels.is_some()) {
      return;
    }

    let (width, height) = handles.size;
    let empty: GraphTexture = bindings.import_view(&mut *graph, "sun shafts none", passes.reflections.get_empty());
    let no_froxels: GraphTexture = bindings.import_view(&mut *graph, "froxels none", passes.froxels.get_empty());
    let (froxel_uniform, froxel_integrated) =
      froxels.unwrap_or_else(|| (runtime.push_uniform(&FroxelUniform::default()), no_froxels));
    let march: SunShaftsParameters = SunShaftsParameters {
      depth_target: handles.depth,
      shadow_maps: lit.shadow_maps,
      shadows: lit.shadows,
      lighting: lit.lighting,
      exposure: StorageValue::new(lit.exposure),
      shafts: empty,
      light_volumes: empty,
      froxels: froxel_uniform,
      froxel_integrated: no_froxels,
      froxel_sampler: passes.froxels.get_sampler(),
    };
    let mut parameters: SunShaftsParameters = SunShaftsParameters {
      light_volumes: light_volumes.unwrap_or(empty),
      froxel_integrated,
      ..march
    };

    if is_shafted {
      let marched: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        "sun shafts marched",
        width.div_ceil(2),
        height.div_ceil(2),
        SunShaftsPass::MARCHED,
      ));

      parameters.shafts = marched;
      graph
        .add_raster_pass("sun shafts march")
        .parameters(&march)
        .color(GraphColorAttachment::new(
          marched,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| passes.sun_shafts.record(context, view, &march, true));
    }

    graph
      .add_raster_pass("sun shafts")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(handles.scene, wgpu::LoadOp::Load))
      .color(GraphColorAttachment::new(handles.high, wgpu::LoadOp::Load))
      .record(move |context| passes.sun_shafts.record(context, view, &parameters, false));
  }
}

impl ViewFeature for SunShaftsFeature {
  /// Its targets are the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}
