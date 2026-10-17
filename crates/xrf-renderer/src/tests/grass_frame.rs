use std::sync::Arc;

use glam::{Mat4, Vec3};
use xrf_renderer_core::{FrameGraph, GraphBindings, GraphCompileOptions, GraphRuntime, GraphTexture, PipelineCache};
use xrf_test_utils::gpu::skip_without_gpu;
use xrf_visual::{DetailsDescription, DetailsModel, DetailsPackage, VisualSection};

use crate::camera::camera_view::CameraView;
use crate::context::gpu_context::GpuContext;
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_grass_settings::RenderGrassSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::grass_card_handles::GrassCardHandles;
use crate::frame::sun_shadow_maps::SunShadowMaps;
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
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::grass_planting::GrassPlanting;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;
use crate::scene::level::grass_view::GrassView;
use crate::scene::texture::texture_cache::TextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_bundle::checkout_dxc;
use crate::tests::test_source::EmptySource;
use crate::tests::test_workers::create_workers;

/// A level's grass of one model, a triangle, over a grid of one cell.
fn create_details() -> RenderLevelDetails {
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
        models: vec![model],
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

/// Plants, culls, stands the cards of, draws and casts into a sun cascade a level's grass for two frames through the
/// frame graph, each pass binding its declared parameters, without a validation error. Skipped, and says so, on a
/// machine with no adapter at all.
#[test]
fn plants_draws_and_casts_the_grass_without_a_validation_error() {
  let context: GpuContext = match GpuContext::create_headless(RenderBackend::D3d12, checkout_dxc().as_deref())
    .or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan, None))
  {
    Ok(context) => context,
    Err(error) => {
      skip_without_gpu(format_args!("no GPU to draw with ({error})"));

      return;
    }
  };
  let device: &wgpu::Device = &context.device;
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let cache: PipelineCache = PipelineCache::new();
  let source: PipelineSource<'_> = PipelineSource {
    cache: &cache,
    device,
    shaders: &shaders,
  };
  let mut textures: TextureCache = TextureCache::new(device, &context.queue, &create_workers());
  let assets: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let view_layout: wgpu::BindGroupLayout = ViewBinding::create_layout(device);
  let view: ViewBinding = ViewBinding::new(device, &view_layout);
  let pass: GrassPass = GrassPass::new(source, &view_layout, textures.get_layout()).unwrap();
  let level: GrassLevel = GrassLevel::new(device, &create_details(), (&mut textures, &assets));
  let targets: ViewTargets = ViewTargets::new(device, 64, 48);
  let pyramid: DepthPyramid = DepthPyramid::new(device, 64, 48);
  let maps: SunShadowMaps = SunShadowMaps::new(device, 64);
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
  let puddles_view: wgpu::TextureView = puddles.create_view(&wgpu::TextureViewDescriptor::default());
  let settings: RenderGrassSettings = RenderGrassSettings {
    is_shadowed: true,
    ..RenderGrassSettings::default()
  };
  let camera: CameraView = CameraView::new(Vec3::new(0.0, 1.0, 0.0), Mat4::IDENTITY, 60.0, 4.0 / 3.0, 0.2, 1000.0);
  let mut grass: GrassView = GrassView::new(device);
  let mut runtime: GraphRuntime = GraphRuntime::new(device, &context.queue);

  for frame in 0..2 {
    grass.prepare(
      device,
      Some(&level),
      &settings,
      &camera,
      0.0,
      (frame as f32 * 0.1, true, 1.0),
      &FoliageWindValues::default(),
      (&puddles_view, GrassPuddlesUniform::default()),
      (false, Some(40.0)),
    );

    {
      let mut graph: FrameGraph<'_> = FrameGraph::new();
      let mut bindings: GraphBindings<'_> = GraphBindings::new();
      let handles: ViewTargetHandles = ViewTargetHandles::import(
        &mut graph,
        &mut bindings,
        (&targets, ViewTargetUses::default(), &targets.light),
      );
      let planting: GrassPlanting = grass
        .add_planting((&mut graph, &mut bindings, &mut runtime), (&pass, Some(&level)))
        .expect("the grass is planted");
      let occlusion: GrassOcclusionParameters = GrassOcclusionParameters {
        pyramid: bindings.import_view(&mut graph, "grass test pyramid", &pyramid.view),
        occlusion: runtime.push_uniform(&<StaticOcclusionUniform as bytemuck::Zeroable>::zeroed()),
      };

      grass.add_cull(&mut graph, (&pass, Some(&level)), (planting, occlusion));

      let cards: Option<GrassCardHandles> =
        grass.add_cards((&mut graph, &mut bindings), (&pass, Some(&level)), planting);

      grass.add_draw(
        &mut graph,
        (&pass, Some(&level)),
        (handles, planting),
        (&view, textures.get_bind_group()),
      );

      let casters: GrassShadowCasters<'_> = grass
        .get_shadow_casters(
          (&mut graph, &mut bindings),
          (&pass, Some(&level), textures.get_bind_group()),
          cards,
        )
        .expect("the cards cast into the cascades");
      let cascades: GraphTexture = bindings.import_view(&mut graph, "grass test cascades", &maps.view);

      casters.add_pass(&mut graph, "grass test cascade", (cascades, 0), &view);

      let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();

      assert!(compiled.get_culled().is_empty(), "culled {:?}", compiled.get_culled());

      let commands: Vec<wgpu::CommandBuffer> = compiled
        .execute((device, &context.queue), &mut runtime, &bindings)
        .unwrap()
        .commands;

      context.queue.submit(commands);
    }

    grass.request_stats();
  }

  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  assert!(pollster::block_on(scope.pop()).is_none());
}
