use glam::Vec4;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture, GraphTextureDescriptor, StorageArray, StorageValue,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_targets::ViewTargets;
use crate::pass::glass_parameters::GlassParameters;
use crate::pass::glass_uniform::GlassUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::scene::level::feature::composited_draw::CompositedDraw;
use crate::scene::level::feature::over_scene::add_over_scene;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_sight::StaticSight;
use crate::scene::static_scene::static_sorted_place::StaticSortedPlace;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;

/// The static scene's composited surfaces over the lit scene: the blended, added and multiplied ones in the cull's
/// order, the strictly sorted ones back to front, the enhanced glass, the distorting surfaces' special pass and the
/// selected ones' mark.
#[derive(Default)]
pub struct CompositedFeature {
  /// The sorted clusters in view, back to front, as `(cluster, place)` entries, and the runs of them alike in layout.
  sorted: Option<wgpu::Buffer>,
  sorted_count: u32,
  sorted_runs: Vec<(StaticLayout, u32, u32)>,
  /// What the enhanced glass draws with, where it draws.
  glass: Option<GlassUniform>,
  /// Whether a distorting surface of the scene is in sight of a lit view.
  is_distorting: bool,
}

impl CompositedFeature {
  /// Takes this frame's composited surfaces through `view`, with what of the scene is in `sight`: the sorted list
  /// written through `encoder`, whether the enhanced glass draws, and whether a distorting surface may.
  pub fn prepare(
    &mut self,
    (scene, device, encoder): (&LevelScene, &wgpu::Device, &mut FrameEncoder),
    view: &CameraView,
    options: &RenderViewOptions,
    sight: &StaticSight,
  ) {
    self.prepare_sorted(scene, device, encoder, view);
    self.is_distorting =
      options.mode.is_lit && !options.mode.is_wireframe && sight.has_pass(StaticSurfacePass::Distortion);
    self.glass = (options.features.glass.is_enhanced()
      && options.mode.is_lit
      && !options.mode.is_wireframe
      && sight.has_pass(StaticSurfacePass::Glass))
    .then_some(GlassUniform {
      refraction: options.features.glass.refraction,
    });
  }

  /// Lists the sorted composited clusters in view, the models' and the level's strictly sorted runs, back to front by
  /// distance and each place's clusters in their parts' order, as the engine draws its sorted blended objects, and the
  /// runs of them alike in layout, which each draw with their layout's program; binds the list for the sorted pass.
  fn prepare_sorted(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    view: &CameraView,
  ) {
    let planes: [Vec4; 6] = view.get_planes();
    let mut places: Vec<(f32, StaticSortedPlace<'_>)> = scene
      .statics
      .list_sorted_places()
      .filter(|place| place.is_in_view(&planes))
      .map(|place| (place.get_distance(view.position), place))
      .collect();

    places.sort_by(|a, b| b.0.total_cmp(&a.0));

    let mut entries: Vec<[u32; 2]> = Vec::new();
    let mut runs: Vec<(StaticLayout, u32, u32)> = Vec::new();

    for (_, place) in &places {
      let first: u32 = entries.len() as u32;

      entries.extend(
        place
          .clusters
          .iter()
          .flat_map(|(first, count)| (*first..first + count).map(|cluster| [cluster, place.place])),
      );

      let count: u32 = entries.len() as u32 - first;

      match runs.last_mut() {
        Some((layout, _, run)) if *layout == place.layout => *run += count,
        _ => runs.push((place.layout, first, count)),
      }
    }

    let bytes: &[u8] = bytemuck::cast_slice(&entries);

    self.sorted_count = entries.len() as u32;
    self.sorted_runs = runs;

    if bytes.is_empty() {
      return;
    }

    if self
      .sorted
      .as_ref()
      .is_none_or(|buffer| buffer.size() < bytes.len() as u64)
    {
      self.sorted = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sorted composited"),
        size: (bytes.len() as u64).next_power_of_two(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
    }

    let Some(buffer) = &self.sorted else {
      return;
    };

    encoder.write_buffer(buffer, 0, bytes);
  }

  /// Whether a distorting surface of the scene is in sight of a lit, solid view.
  pub fn is_distorting(&self) -> bool {
    self.is_distorting
  }

  /// Adds the blended, added and multiplied surfaces in the cull's order, over the lit scene.
  pub fn add_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    (draw, sky): (&CompositedDraw<'a>, SkyParameters<'a>),
  ) {
    let CompositedDraw {
      view,
      textures,
      layouts,
      composited,
      ..
    } = *draw;
    let args: Vec<GraphBuffer> = draw.args.clone();
    let builder = args.iter().fold(
      StaticDraws::declare_layouts(add_over_scene(&mut *graph, draw.handles, "composited"), &layouts),
      |builder, args| builder.buffer(*args, GraphBufferAccess::Indirect),
    );

    builder.parameters(&composited).parameters(&sky).record(move |context| {
      let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

      passes
        .composited
        .record(context, (view, &layouts, textures), (&composited, &sky), &args);
    });
  }

  /// Adds what draws after the particles: the sorted surfaces back to front, as the engine draws `mapSorted` after its
  /// unsorted forward passes; the enhanced glass over the frame copied just before it, which it refracts; and the
  /// distorting surfaces' special pass, a window's glass bending what is behind it, into the distortion target the
  /// water and the particles wrote, as the engine draws `mapDistort` after every forward pass.
  pub fn add_after_particles<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (draw, sky, lit): (&CompositedDraw<'a>, SkyParameters<'a>, LightingHandles<'a>),
  ) {
    let CompositedDraw {
      view,
      textures,
      handles,
      statics,
      layouts,
      composited,
      ..
    } = *draw;

    if let Some(list) = self.sorted.as_ref().filter(|_| self.sorted_count > 0) {
      let list: GraphBuffer = bindings.import_buffer(&mut *graph, "sorted composited", list);
      let sorted: [StaticDrawParameters; StaticLayout::COUNT] = statics.get_layout_draws(StorageArray::new(list));
      let runs: &'a [(StaticLayout, u32, u32)] = &self.sorted_runs;

      StaticDraws::declare_layouts(add_over_scene(&mut *graph, handles, "sorted composited"), &sorted)
        .parameters(&composited)
        .parameters(&sky)
        .record(move |context| {
          passes
            .composited
            .record_sorted(context, (view, &sorted, textures), (&composited, &sky), runs);
        });
    }

    if let Some(glass) = self.glass {
      let (width, height) = handles.size;
      let copy: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        "glass scene",
        width,
        height,
        ViewTargets::SCENE,
      ));

      add_texture_copy(&mut *graph, "glass copy", handles.scene, copy);

      let parameters: GlassParameters = GlassParameters {
        lighting: lit.lighting,
        exposure: StorageValue::new(lit.exposure),
        material_lut: lit.material_lut,
        lut_sampler: lit.lut_sampler,
        shadow_maps: lit.shadow_maps,
        shadows: lit.shadows,
        glass: runtime.push_uniform(&glass),
        glass_scene: copy,
        scene_sampler: passes.glass.get_scene_sampler(),
        depth_target: handles.depth,
      };
      let args: Vec<GraphBuffer> = draw.args.clone();
      let builder = args.iter().fold(
        StaticDraws::declare_layouts(add_over_scene(&mut *graph, handles, "glass"), &layouts),
        |builder, args| builder.buffer(*args, GraphBufferAccess::Indirect),
      );

      builder.parameters(&parameters).parameters(&sky).record(move |context| {
        let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

        passes
          .glass
          .record(context, (view, &layouts, textures), (&parameters, &sky), &args);
      });
    }

    if self.is_distorting {
      let args: Vec<GraphBuffer> = draw.args.clone();
      let pass = graph
        .add_raster_pass("composited distortion")
        .color(GraphColorAttachment::new(handles.distortion, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(handles.depth));
      let builder = args
        .iter()
        .fold(StaticDraws::declare_layouts(pass, &layouts), |builder, args| {
          builder.buffer(*args, GraphBufferAccess::Indirect)
        });

      builder.parameters(&composited).parameters(&sky).record(move |context| {
        let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

        passes
          .composited
          .record_distortion(context, (view, &layouts, textures), (&composited, &sky), &args);
      });
    }
  }

  /// Adds the selected surfaces' mark into the material target, where the present outlines the selection.
  pub fn add_selection_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    draw: &CompositedDraw<'a>,
  ) {
    let CompositedDraw {
      view,
      textures,
      handles,
      layouts,
      ..
    } = *draw;
    let args: Vec<GraphBuffer> = draw.args.clone();
    let pass = graph
      .add_raster_pass("composited selection")
      .color(GraphColorAttachment::new(handles.material, wgpu::LoadOp::Load))
      .depth(GraphDepthAttachment::new_read_only(handles.depth));
    let builder = args
      .iter()
      .fold(StaticDraws::declare_layouts(pass, &layouts), |builder, args| {
        builder.buffer(*args, GraphBufferAccess::Indirect)
      });

    builder.record(move |context| {
      let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

      passes
        .composited
        .record_selection(context, (view, &layouts, textures), &args);
    });
  }
}

impl ViewFeature for CompositedFeature {
  /// The sorted list's buffer, grown to the most sorted clusters seen in view.
  fn get_memory(&self) -> u64 {
    self.sorted.as_ref().map_or(0, wgpu::Buffer::size)
  }

  fn release(&mut self) {
    self.sorted = None;
    self.sorted_count = 0;
    self.sorted_runs.clear();
  }
}
