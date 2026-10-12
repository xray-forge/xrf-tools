use glam::Vec4;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphDepthAttachment, GraphRuntime, GraphTexture,
  GraphTextureDescriptor, StorageArray, UniformBinding,
};

use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::froxel_volume::FroxelVolume;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::froxel_inject_parameters::FroxelInjectParameters;
use crate::pass::froxel_integrate_parameters::FroxelIntegrateParameters;
use crate::pass::froxel_uniform::FroxelUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_volume_parameters::LightVolumeParameters;
use crate::pass::light_volume_pass::LightVolumePass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::lights_feature::LightsFeature;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The volumetric lighting: the enhanced froxels, a grid along the view filled over its history and integrated, in
/// place of the sun's and the lights' shafts; or the shadowed lights' shafts (`accum_volumetric`) the sun shafts'
/// composite adds.
#[derive(Default)]
pub struct VolumetricFeature {
  /// What the froxels read this frame, while they draw.
  uniform: Option<FroxelUniform>,
  froxels: Option<FroxelVolume>,
}

impl VolumetricFeature {
  /// Takes this frame's volumetric lighting: the froxels made or kept where the enhanced volume draws, which is lit and
  /// solid, dropped where it does not.
  pub fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions) {
    let settings = options.features.volumetric;

    if !(settings.is_enhanced() && options.mode.is_lit && !options.mode.is_wireframe) {
      self.uniform = None;
      self.froxels = None;

      return;
    }

    let volume: &FroxelVolume = self.froxels.get_or_insert_with(|| FroxelVolume::new(device));
    let [width, height, depth] = FroxelVolume::SIZE;
    // Each frame's points a golden share further along their slices, so the history gathers the whole of each.
    let jitter: f32 = (volume.histories.get_frame() as f32 * 0.618_034).fract();

    self.uniform = Some(FroxelUniform {
      size: Vec4::new(width as f32, height as f32, depth as f32, settings.distance),
      params: Vec4::new(
        settings.intensity,
        settings.anisotropy.clamp(0.0, 0.95),
        jitter,
        if volume.histories.is_valid() { 1.0 } else { 0.0 },
      ),
    });
  }

  /// Whether the froxels draw this frame, in place of the sun's and the lights' shafts.
  pub fn is_froxeled(&self) -> bool {
    self.uniform.is_some() && self.froxels.is_some()
  }

  /// Adds the shadowed lights' shafts at half the frame's size, added up over the frame's depth for the sun shafts'
  /// composite; answers them, none where the froxels draw instead or no light draws shafts.
  pub fn add_light_volume_passes<'a>(
    &self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (view, textures): (&'a ViewBinding, &'a wgpu::BindGroup),
    (handles, lights, atlas): (ViewTargetHandles, &'a LightsFeature, &'a wgpu::TextureView),
  ) -> Option<GraphTexture> {
    if self.is_froxeled() || lights.volumes.is_empty() {
      return None;
    }

    let (width, height) = (handles.size.0.div_ceil(2), handles.size.1.div_ceil(2));
    let target: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "light volumes",
      width,
      height,
      LightVolumePass::FORMAT,
    ));
    let depth: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "light volumes depth",
      width,
      height,
      ViewTargets::DEPTH,
    ));
    let parameters: LightVolumeParameters = LightVolumeParameters {
      light_volumes: StorageArray::new(bindings.import_buffer(&mut *graph, "light volumes", &lights.volume_buffer)),
      records: StorageArray::new(bindings.import_buffer(&mut *graph, "light records", &lights.record_buffer)),
      shadow_atlas: bindings.import_view(&mut *graph, "light shadow atlas", atlas),
      shadow_sampler: passes.light_volumes.get_shadow_sampler(),
      volumes: runtime.push_uniform(&lights.volumes_uniform),
      depth_target: handles.depth,
    };
    let slices: &'a [u32] = &lights.volume_slices;

    graph
      .add_raster_pass("light volumes depth")
      .parameters(&parameters)
      .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0)))
      .record(move |context| passes.light_volumes.record_depth(context, view, &parameters, textures));
    graph
      .add_raster_pass("light volumes")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(
        target,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .depth(GraphDepthAttachment::new_read_only(depth))
      .record(move |context| {
        passes
          .light_volumes
          .record(context, view, &parameters, textures, slices)
      });

    Some(target)
  }

  /// Fills the froxels over their history and integrates them along the view; answers the grid's uniform and what was
  /// integrated, for the sun shafts' composite to read in place of both shafts, none where they do not draw.
  pub fn add_froxel_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (lit, atlas, lights): (LightingHandles<'a>, &'a wgpu::TextureView, &'a LightsFeature),
  ) -> Option<(UniformBinding<FroxelUniform>, GraphTexture)> {
    let (Some(uniform), Some(volume)) = (&self.uniform, &self.froxels) else {
      return None;
    };
    let froxels: UniformBinding<FroxelUniform> = runtime.push_uniform(uniform);
    let history: GraphTexture = bindings.import_view(graph, "froxel history read", volume.histories.get_read());
    let current: GraphTexture = bindings.import_view(graph, "froxel history write", volume.histories.get_written());
    let integrated: GraphTexture = bindings.import_view(graph, "froxels integrated", &volume.integrated);
    let inject: FroxelInjectParameters<'a> = FroxelInjectParameters {
      lighting: lit.lighting,
      shadow_maps: lit.shadow_maps,
      shadows: lit.shadows,
      records: StorageArray::new(bindings.import_buffer(graph, "light records", &lights.record_buffer)),
      counts: StorageArray::new(bindings.import_buffer(graph, "light cluster counts", &lights.counts)),
      items: StorageArray::new(bindings.import_buffer(graph, "light cluster items", &lights.items)),
      lights: runtime.push_uniform(&lights.uniform),
      shadow_atlas: bindings.import_view(graph, "light shadow atlas", atlas),
      froxels,
      froxel_history: history,
      froxel_sampler: passes.froxels.get_sampler(),
      froxel_current: current,
    };
    let integrate: FroxelIntegrateParameters = FroxelIntegrateParameters {
      froxels,
      froxel_scattered: current,
      froxel_integrated: integrated,
    };

    graph
      .add_compute_pass("froxels")
      .parameters(&inject)
      .record(move |context| passes.froxels.record_inject(context, view, &inject));
    graph
      .add_compute_pass("froxels integrate")
      .parameters(&integrate)
      .record(move |context| passes.froxels.record_integrate(context, view, &integrate));

    Some((froxels, integrated))
  }

  /// The froxels this frame wrote become the history the next reads, while they draw.
  pub fn end_frame(&mut self) {
    if let (Some(_), Some(froxels)) = (self.uniform, &mut self.froxels) {
      froxels.histories.swap();
    }
  }
}

impl ViewFeature for VolumetricFeature {
  /// The froxels' grids, while they draw.
  fn get_memory(&self) -> u64 {
    self.froxels.as_ref().map_or(0, FroxelVolume::get_memory)
  }

  fn release(&mut self) {
    self.uniform = None;
    self.froxels = None;
  }
}
