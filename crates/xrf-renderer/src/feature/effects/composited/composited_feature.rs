use glam::Mat4;
use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphBufferDescriptor, GraphColorAttachment, GraphDeclaration,
  GraphDepthAttachment, GraphTexture, GraphTextureDescriptor, StorageArray, StorageArrayMut, StorageValue,
};

use crate::camera::camera_frustum::CameraFrustum;
use crate::camera::camera_view::CameraView;
use crate::feature::effects::composited::composited_draw::CompositedDraw;
use crate::feature::effects::composited::glass::glass_copy_dispatch_parameters::GlassCopyDispatchParameters;
use crate::feature::effects::composited::glass::glass_copy_parameters::GlassCopyParameters;
use crate::feature::effects::composited::glass::glass_parameters::GlassParameters;
use crate::feature::effects::composited::glass::glass_uniform::GlassUniform;
use crate::feature::effects::over_scene::add_over_scene;
use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::feature::scene::statics::draw::static_draws::StaticDraws;
use crate::feature::scene::statics::draw::static_layout_bindings::StaticLayoutBindings;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::jitter_guard::JitterGuard;
use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::view::record::lighting_handles::LightingHandles;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::static_scene::draw::static_sight::StaticSight;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::item::static_sorted_place::StaticSortedPlace;
use crate::scene::static_scene::surface::static_surface_pass::StaticSurfacePass;

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
  /// The tiles of the frame the distorting surfaces in sight of a lit view may touch, by the spheres about them.
  distortion_tiles: ScreenTiles,
}

impl CompositedFeature {
  /// Takes this frame's composited surfaces through the camera, with what of the `scene` is in `sight`: the sorted list
  /// written through `encoder`, whether the enhanced glass draws, and where on the frame a distorting surface may.
  pub fn prepare(
    &mut self,
    &ViewPrepare {
      device, view, options, ..
    }: &ViewPrepare<'_>,
    scene: &LevelScene,
    encoder: &mut FrameEncoder,
    sight: &StaticSight,
  ) {
    self.prepare_sorted(scene, device, encoder, view);
    self.distortion_tiles = ScreenTiles::default();

    if options.mode.is_lit && !options.mode.is_wireframe && sight.has_pass(StaticSurfacePass::Distortion) {
      let (frustum, view_projection): (CameraFrustum, Mat4) = (view.get_frustum(), view.get_view_projection());

      for sphere in scene
        .statics
        .list_seen_spheres(&frustum, |sight| sight.has_pass(StaticSurfacePass::Distortion))
      {
        self.distortion_tiles.cover_sphere(view_projection, sphere);
      }
    }

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
    let frustum: CameraFrustum = view.get_frustum();
    let mut places: Vec<(f32, StaticSortedPlace<'_>)> = scene
      .statics
      .list_sorted_places()
      .filter(|place| frustum.reaches(place.sphere))
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

  /// The tiles of the frame a distorting surface of the scene in sight of a lit, solid view may touch.
  pub fn get_distortion_tiles(&self) -> &ScreenTiles {
    &self.distortion_tiles
  }

  /// Adds the blended, added and multiplied surfaces in the cull's order, over the lit scene.
  pub fn add_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    draw: &CompositedDraw<'a>,
    sky: SkyParameters<'a>,
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

      passes.effects.composited.record(
        context,
        StaticLayoutBindings::new(view, &layouts, textures),
        &composited,
        &sky,
        &args,
      );
    });
  }

  /// Adds what draws after the particles: the sorted surfaces back to front, as the engine draws `mapSorted` after its
  /// unsorted forward passes; the enhanced glass over the frame copied just before it, which it refracts; and the
  /// distorting surfaces' special pass, a window's glass bending what is behind it, into the distortion target the
  /// water and the particles wrote, as the engine draws `mapDistort` after every forward pass.
  pub fn add_after_particles<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    draw: &CompositedDraw<'a>,
    sky: SkyParameters<'a>,
    lit: LightingHandles<'a>,
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
          passes.effects.composited.record_sorted(
            context,
            StaticLayoutBindings::new(view, &sorted, textures),
            &composited,
            &sky,
            runs,
          );
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

      // Taken only where the cull lists glass to draw, early or late: the dispatch is worked out on the GPU.
      let args: Vec<GraphBuffer> = draw.args.clone();
      let dispatch: GraphBuffer = graph.create_buffer(GraphBufferDescriptor::new("glass copy dispatch", 12));
      let counted: GlassCopyDispatchParameters = GlassCopyDispatchParameters {
        early: StorageArray::new(args[0]),
        late: StorageArray::new(args.get(1).copied().unwrap_or(args[0])),
        scene: handles.scene,
        dispatch: StorageArrayMut::new(dispatch),
      };
      let copied: GlassCopyParameters = GlassCopyParameters {
        scene: handles.scene,
        copied: copy,
      };

      // Written whole by the dispatch's pass, cleared first so the graph sees it written before that pass reads it.
      graph
        .add_encoder_pass("glass copy dispatch clear")
        .buffer(dispatch, GraphBufferAccess::CopyDestination)
        .record(move |context| {
          let dispatch: &wgpu::Buffer = context.get_buffer(dispatch);

          context.get_encoder().clear_buffer(dispatch, 0, None);
        });
      graph
        .add_compute_pass("glass copy dispatch")
        .parameters(&counted)
        .record(move |context| passes.effects.glass.record_copy_dispatch(context, &counted));
      graph
        .add_compute_pass("glass copy")
        .parameters(&copied)
        .buffer(dispatch, GraphBufferAccess::Indirect)
        .record(move |context| {
          let dispatch: &wgpu::Buffer = context.get_buffer(dispatch);

          passes.effects.glass.record_copy(context, &copied, dispatch);
        });

      let parameters: GlassParameters<'_> = GlassParameters {
        lighting: lit.lighting,
        exposure: StorageValue::new(lit.exposure),
        material_lut: lit.material_lut,
        lut_sampler: lit.lut_sampler,
        shadow_maps: lit.shadow_maps,
        shadows: lit.shadows,
        glass: runtime.push_uniform(&glass),
        glass_scene: copy,
        scene_sampler: passes.effects.glass.get_scene_sampler(),
        depth_target: handles.depth,
      };
      let builder = args.iter().fold(
        StaticDraws::declare_layouts(add_over_scene(&mut *graph, handles, "glass"), &layouts),
        |builder, args| builder.buffer(*args, GraphBufferAccess::Indirect),
      );

      builder.parameters(&parameters).parameters(&sky).record(move |context| {
        let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

        passes.effects.glass.record(
          context,
          StaticLayoutBindings::new(view, &layouts, textures),
          &parameters,
          &sky,
          &args,
        );
      });
    }

    if !self.distortion_tiles.is_empty() {
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

        passes.effects.composited.record_distortion(
          context,
          StaticLayoutBindings::new(view, &layouts, textures),
          &composited,
          &sky,
          &args,
        );
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
    // The selected glass marked for the present's outline: an editor overlay, no light.
    let pass = graph
      .add_raster_pass("composited selection")
      .tag(JitterGuard::READS_DEPTH)
      .tag(JitterGuard::READS_MATERIAL)
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
        .effects
        .composited
        .record_selection(context, StaticLayoutBindings::new(view, &layouts, textures), &args);
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
