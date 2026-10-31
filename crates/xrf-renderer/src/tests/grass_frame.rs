use std::sync::Arc;

use glam::{Mat4, Vec3};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphCompileOptions, GraphDeclaration, GraphRuntime, GraphTexture, PipelineCache,
};
use xrf_visual::{DetailsDescription, DetailsModel, DetailsPackage, VisualSection};

use crate::camera::camera_view::CameraView;
use crate::context::gpu_context::GpuContext;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_target_uses::ViewTargetUses;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_details::RenderLevelDetails;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::pass::grass_pass::GrassPass;
use crate::pass::grass_puddles_uniform::GrassPuddlesUniform;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::grass_card_handles::GrassCardHandles;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::grass_planting::GrassPlanting;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;
use crate::scene::level::grass_view::GrassView;
use crate::scene::level::grass_view_frame::GrassViewFrame;
use crate::scene::level::level_grass::LevelGrass;
use crate::scene::level::sun_shadow_maps::SunShadowMaps;
use crate::scene::texture::texture_cache::TextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_context::create_test_context;
use crate::tests::test_source::EmptySource;
use crate::tests::test_workers::create_workers;

/// A level's grass of `models` models, each the same triangle, over a grid of one cell.
fn create_details(models: usize) -> RenderLevelDetails {
  let positions: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
  let uvs: [f32; 6] = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
  let indices: [u16; 3] = [0, 1, 2];
  let grid: [u32; 1] = [0];
  let mut buffer: Vec<u8> = Vec::new();
  let mut section = |bytes: &[u8]| -> VisualSection {
    let byte_offset: u32 = buffer.len() as u32;

    buffer.extend_from_slice(bytes);

    VisualSection {
      byte_offset,
      byte_length: bytes.len() as u32,
    }
  };
  let positions: VisualSection = section(bytemuck::cast_slice(&positions));
  let uvs: VisualSection = section(bytemuck::cast_slice(&uvs));
  let grid: VisualSection = section(bytemuck::cast_slice(&grid));
  let model: DetailsModel = DetailsModel {
    texture: String::from("detail\\grass"),
    is_waving: true,
    min_scale: 0.5,
    max_scale: 1.0,
    height: 1.0,
    radius: 1.0,
    positions,
    uvs,
    indices: section(bytemuck::cast_slice(&indices)),
  };
  let empty: VisualSection = VisualSection {
    byte_offset: buffer.len() as u32,
    byte_length: 0,
  };
  let buffer_length: u32 = buffer.len() as u32;

  RenderLevelDetails {
    package: DetailsPackage {
      description: DetailsDescription {
        size_x: 1,
        size_z: 1,
        offset_x: 0,
        offset_z: 0,
        models: vec![model; models],
        grid,
        slots: empty,
        slot_count: 0,
        bins: empty,
        triangles: empty,
        buffer_length,
      },
      buffer,
    },
    surfaces: Vec::new(),
  }
}

/// What a view's grass is drawn with here: the pass, one view's targets, depth pyramid and sun cascades, the puddle
/// sites it plants around, and the textures its models sample.
struct GrassFrames {
  textures: TextureCache,
  assets: Arc<dyn RenderAssetSource>,
  view: ViewBinding,
  pass: GrassPass,
  targets: ViewTargets,
  pyramid: DepthPyramid,
  maps: SunShadowMaps,
  _puddles: wgpu::Texture,
  puddles_view: wgpu::TextureView,
  settings: RenderGrassSettings,
  camera: CameraView,
  runtime: GraphRuntime,
}

impl GrassFrames {
  fn new(context: &GpuContext) -> Self {
    let device: &wgpu::Device = &context.device;
    let shaders: ShaderLibrary = ShaderLibrary::default();
    let cache: PipelineCache = PipelineCache::new();
    let source: PipelineSource<'_> = PipelineSource {
      cache: &cache,
      device,
      shaders: &shaders,
    };
    let textures: TextureCache = TextureCache::new(device, &context.queue, &create_workers());
    let view_layout: wgpu::BindGroupLayout = ViewBinding::create_layout(device);
    let puddles: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("grass test puddles"),
      size: wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba32Float,
      usage: wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });

    Self {
      assets: Arc::new(EmptySource),
      view: ViewBinding::new(device, &view_layout),
      pass: GrassPass::new(source, &view_layout, textures.get_layout()).unwrap(),
      textures,
      targets: ViewTargets::new(device, 64, 48),
      pyramid: DepthPyramid::new(device, 64, 48),
      maps: SunShadowMaps::new(device, 64),
      puddles_view: puddles.create_view(&wgpu::TextureViewDescriptor::default()),
      _puddles: puddles,
      settings: RenderGrassSettings {
        is_shadowed: true,
        ..RenderGrassSettings::default()
      },
      camera: CameraView::new(Vec3::new(0.0, 1.0, 0.0), Mat4::IDENTITY, 60.0, 4.0 / 3.0, 0.2, 1000.0),
      runtime: GraphRuntime::new(device, &context.queue),
    }
  }

  /// Plants, culls, stands the cards of, draws and casts into a sun cascade `level`'s grass for one frame of `grass`
  /// through the frame graph, every pass declared, and submits it.
  fn draw(&mut self, context: &GpuContext, grass: &mut GrassView, level: &GrassLevel, frame: u32) {
    let device: &wgpu::Device = &context.device;

    grass.prepare(
      device,
      Some(level),
      &self.settings,
      &GrassViewFrame {
        camera: &self.camera,
        discard_below: 0.0,
        time: frame as f32 * 0.1,
        is_windy: true,
        strength: 1.0,
        foliage: &FoliageWindValues::default(),
        puddles: (&self.puddles_view, GrassPuddlesUniform::default()),
        is_traced: false,
        cascade_reach: Some(40.0),
      },
    );

    {
      let (pass, textures): (&GrassPass, &TextureCache) = (&self.pass, &self.textures);
      let mut graph: FrameGraph<'_> = FrameGraph::new();
      let mut bindings: GraphBindings<'_> = GraphBindings::new();
      let handles: ViewTargetHandles = ViewTargetHandles::import(
        &mut graph,
        &mut bindings,
        (&self.targets, ViewTargetUses::default(), &self.targets.light),
      );
      let planting: GrassPlanting = grass
        .add_planting(
          GraphDeclaration::new(&mut graph, &mut bindings, &mut self.runtime),
          (pass, Some(level)),
        )
        .expect("the grass is planted");
      let occlusion: GrassOcclusionParameters = GrassOcclusionParameters {
        pyramid: bindings.import_view(&mut graph, "grass test pyramid", &self.pyramid.view),
        occlusion: self
          .runtime
          .push_uniform(&<StaticOcclusionUniform as bytemuck::Zeroable>::zeroed()),
      };

      grass.add_cull(&mut graph, (pass, Some(level)), (planting, occlusion));

      let cards: Option<GrassCardHandles> = grass.add_cards(
        GraphDeclaration::new(&mut graph, &mut bindings, &mut self.runtime),
        (pass, Some(level)),
        planting,
      );

      grass.add_draw(
        &mut graph,
        (pass, Some(level)),
        (handles, planting),
        (&self.view, textures.get_bind_group()),
      );

      let casters: GrassShadowCasters<'_> = grass
        .get_shadow_casters(
          GraphDeclaration::new(&mut graph, &mut bindings, &mut self.runtime),
          (pass, Some(level), textures.get_bind_group()),
          cards,
        )
        .expect("the cards cast into the cascades");
      let cascades: GraphTexture = bindings.import_view(&mut graph, "grass test cascades", &self.maps.view);

      casters.add_pass(&mut graph, "grass test cascade", (cascades, 0), &self.view);

      let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

      assert!(compiled.get_culled().is_empty(), "culled {:?}", compiled.get_culled());

      let commands: Vec<wgpu::CommandBuffer> = compiled
        .execute((device, &context.queue), &mut self.runtime, &bindings)
        .unwrap()
        .commands;

      context.queue.submit(commands);
    }

    grass.request_stats();
  }
}

/// Plants, culls, stands the cards of, draws and casts into a sun cascade a level's grass for two frames through the
/// frame graph, each pass binding its declared parameters, without a validation error. Skipped, and says so, on a
/// machine with no adapter at all.
#[test]
fn plants_draws_and_casts_the_grass_without_a_validation_error() {
  let Some(context) = create_test_context("to draw with") else {
    return;
  };
  let scope: wgpu::ErrorScopeGuard = context.device.push_error_scope(wgpu::ErrorFilter::Validation);
  let mut frames: GrassFrames = GrassFrames::new(&context);
  let level: GrassLevel = GrassLevel::new(
    &context.device,
    &create_details(1),
    (&mut frames.textures, &frames.assets),
  );
  let mut grass: GrassView = GrassView::new(&context.device);

  for frame in 0..2 {
    frames.draw(&context, &mut grass, &level, frame);
  }

  context.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  assert!(pollster::block_on(scope.pop()).is_none());
}

/// A scene's grass replaced by another read is planted again whole by a view drawing it, every pass binding the new
/// level's buffers, without a validation error; the same level drawn again plants nothing anew, and the scene's grass
/// posted as none leaves nothing to draw.
#[test]
fn a_replaced_grass_is_planted_again_and_drawn_without_a_validation_error() {
  let Some(context) = create_test_context("to draw with") else {
    return;
  };
  let scope: wgpu::ErrorScopeGuard = context.device.push_error_scope(wgpu::ErrorFilter::Validation);
  let mut frames: GrassFrames = GrassFrames::new(&context);
  let mut scene: LevelGrass = LevelGrass::default();
  let mut grass: GrassView = GrassView::new(&context.device);
  let mut generations: Vec<u32> = Vec::new();

  for (frame, models) in [1, 1, 3, 3].into_iter().enumerate() {
    if frame % 2 == 0 {
      scene.replace(
        &context.device,
        Some(create_details(models)),
        (&mut frames.textures, &frames.assets),
      );
    }

    let level: &GrassLevel = scene.get_level().expect("the grass is posted");

    assert_eq!(level.model_count, models as u32);
    frames.draw(&context, &mut grass, level, frame as u32);
    assert_eq!(grass.get_trace_uniform().model_count, models as u32);
    generations.push(grass.get_generation());
  }

  assert_eq!(generations[0], generations[1], "the same grass is not planted again");
  assert!(generations[2] > generations[1], "a replaced grass is planted again");
  assert_eq!(generations[2], generations[3]);

  scene.replace(&context.device, None, (&mut frames.textures, &frames.assets));

  assert!(scene.get_level().is_none());

  context.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  assert!(pollster::block_on(scope.pop()).is_none());
}
