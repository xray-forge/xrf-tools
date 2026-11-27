use xrf_renderer_core::{FrameGraph, GraphBindings, GraphCompileOptions, GraphRuntime, GraphTexture, StorageArray};

use crate::feature::effects::rain::draw::rain_parameters::RainParameters;
use crate::feature::effects::rain::rain_uniform::RainUniform;
use crate::feature::effects::rain::rings::rain_rings::RainRings;
use crate::feature::effects::rain::rings::rain_rings_draw::RainRingsDraw;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::support::test_bundle::TestBundle;
use crate::tests::support::test_context::create_test_context;
use crate::tests::support::test_gpu::build_test_gpu;
use crate::thread::gpu::gpu_state::GpuState;

/// One texel of `format`, standing for a map the rain's parameters bind.
fn create_texel(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::TextureView {
  device
    .create_texture(&wgpu::TextureDescriptor {
      label: Some("rain rings test texel"),
      size: wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format,
      usage: wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    })
    .create_view(&Default::default())
}

// The rings are one texture of several levels, drawn into its first and reduced into the rest: the graph compiles the
// two passes and the device takes them.
#[test]
fn the_rings_draw_into_their_first_level_and_reduce_into_the_rest() {
  let Some(context) = create_test_context("to draw with") else {
    return;
  };
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let gpu: GpuState = build_test_gpu(context, &shaders, &TestBundle::checkout()).unwrap();
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&gpu.context.device, &gpu.context.queue);
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let passes: LevelPasses<'_> = gpu.get_level_passes(&shaders);
  let view: ViewBinding = ViewBinding::new(device, &gpu.view_layout);
  let rings: RainRings = RainRings::new(device, RainRings::get_size(15.0));
  let depth: wgpu::TextureView = create_texel(device, wgpu::TextureFormat::Depth32Float);
  let heights: wgpu::TextureView = create_texel(device, wgpu::TextureFormat::R32Float);
  let picture: wgpu::TextureView = create_texel(device, wgpu::TextureFormat::Rgba8Unorm);
  let model: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("rain rings test model"),
    size: 256,
    usage: wgpu::BufferUsages::STORAGE,
    mapped_at_creation: false,
  });
  let mut runtime: GraphRuntime = GraphRuntime::new(device, queue);

  {
    let mut graph: FrameGraph<'_> = FrameGraph::new();
    let mut bindings: GraphBindings<'_> = GraphBindings::new();
    let levels: [GraphTexture; RainRings::LEVELS as usize] =
      std::array::from_fn(|level| bindings.import_view(&mut graph, "rain rings level", &rings.level_views[level]));
    let parameters: RainParameters<'_> = RainParameters {
      rain: runtime.push_uniform(&RainUniform::default()),
      cover: bindings.import_view(&mut graph, "rain rings test cover", &depth),
      surface_water: bindings.import_view(&mut graph, "rain rings test water", &depth),
      surface_lowest: bindings.import_view(&mut graph, "rain rings test lowest", &heights),
      puddles: bindings.import_view(&mut graph, "rain rings test puddles", &heights),
      streak_texture: bindings.import_view(&mut graph, "rain rings test streak", &picture),
      splash_texture: bindings.import_view(&mut graph, "rain rings test splash", &picture),
      rain_sampler: passes.effects.rain.get_sampler(),
      splash_vertices: StorageArray::new(bindings.import_buffer(&mut graph, "rain rings test vertices", &model)),
      splash_indices: StorageArray::new(bindings.import_buffer(&mut graph, "rain rings test indices", &model)),
    };

    passes.effects.rain.add_rings(
      &mut graph,
      &view,
      parameters,
      RainRingsDraw {
        levels,
        size: rings.size,
        near: 4,
      },
    );

    let commands: Vec<wgpu::CommandBuffer> = graph
      .compile(&GraphCompileOptions::default())
      .unwrap()
      .execute((device, queue), &mut runtime, &bindings)
      .unwrap()
      .commands;

    queue.submit(commands);
  }

  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  assert!(pollster::block_on(scope.pop()).is_none());
}
